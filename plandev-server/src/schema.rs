//! Builds a Hasura-shaped GraphQL schema from the UI contract.
//!
//! Skeleton stage: every root field the UI uses exists with the argument types
//! the UI declares and a return shape unioned from every selection the UI
//! makes. Resolvers return empty data. Real behaviour replaces resolvers per
//! table as milestones land.

use async_graphql::dynamic::*;
use async_graphql::parser::types::{
    ExecutableDocument, OperationType, Selection, SelectionSet, Type as GType, BaseType,
};
use async_graphql::{Name, Value};
use plandev_contract::{operations, Kind};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Default, Debug)]
struct Node {
    children: BTreeMap<String, Node>,
    /// arg name -> declared type text (first seen wins); only meaningful on a field's own node
    args: BTreeMap<String, String>,
}

#[derive(Default)]
struct RootInfo {
    tree: Node,
}

const BUILTIN: [&str; 5] = ["Int", "Float", "String", "Boolean", "ID"];

fn collect(set: &SelectionSet, doc: &ExecutableDocument, vars: &BTreeMap<String, String>, into: &mut Node) {
    for sel in &set.items {
        match &sel.node {
            Selection::Field(f) => {
                let child = into.children.entry(f.node.name.node.to_string()).or_default();
                record_args(f.node.arguments.iter().map(|(n, v)| (n.node.to_string(), v.node.to_string())), vars, &mut child.args);
                collect(&f.node.selection_set.node, doc, vars, child);
            }
            Selection::InlineFragment(i) => collect(&i.node.selection_set.node, doc, vars, into),
            Selection::FragmentSpread(s) => {
                if let Some(frag) = doc.fragments.get(&s.node.fragment_name.node) {
                    collect(&frag.node.selection_set.node, doc, vars, into);
                }
            }
        }
    }
}

fn record_args(
    arguments: impl Iterator<Item = (String, String)>,
    vars: &BTreeMap<String, String>,
    into: &mut BTreeMap<String, String>,
) {
    for (arg_name, text) in arguments {
        let ty = text
            .strip_prefix('$')
            .and_then(|v| vars.get(v))
            .map(|s| s.trim_end_matches('!').to_string());
        into.entry(arg_name).or_insert_with(|| ty.unwrap_or_else(|| "Json".into()));
    }
}

fn arg_inputs(args: &BTreeMap<String, String>) -> Vec<InputValue> {
    args.iter()
        .map(|(n, t)| {
            let tr = GType::new(t).map(|p| type_ref(&p)).unwrap_or(TypeRef::named("Json"));
            InputValue::new(n.clone(), tr)
        })
        .collect()
}

fn type_ref(t: &GType) -> TypeRef {
    let inner = match &t.base {
        BaseType::Named(n) => TypeRef::Named(n.to_string().into()),
        BaseType::List(l) => TypeRef::List(Box::new(type_ref(l))),
    };
    if t.nullable { inner } else { TypeRef::NonNull(Box::new(inner)) }
}

fn named_types(t: &GType, out: &mut BTreeSet<String>) {
    match &t.base {
        BaseType::Named(n) => {
            out.insert(n.to_string());
        }
        BaseType::List(l) => named_types(l, out),
    }
}

/// Is this root field a single object (vs a list)?
fn root_is_single(field: &str) -> bool {
    field.ends_with("_by_pk")
        || field.ends_with("_one")
        || field.ends_with("_aggregate")
        || field.chars().any(|c| c.is_uppercase())
        || field.starts_with("delete_") && false
}

/// Mutation responses (`insert_x`, `update_x`, `delete_x` without _one/_by_pk).
fn is_mutation_response(field: &str) -> bool {
    (field.starts_with("insert_") || field.starts_with("update_") || field.starts_with("delete_"))
        && !field.ends_with("_one")
        && !field.ends_with("_by_pk")
}

fn nested_is_list(name: &str) -> bool {
    matches!(name, "returning" | "nodes") || (name.ends_with('s') && !name.ends_with("ss") && name != "status")
}

fn leaf_value(name: &str) -> Value {
    match name {
        "count" | "affected_rows" => Value::from(0),
        _ => Value::Null,
    }
}

fn build_object(type_name: &str, node: &Node, objs: &mut Vec<Object>) {
    let mut obj = Object::new(type_name);
    for (name, child) in &node.children {
        let fname = name.clone();
        if child.children.is_empty() {
            let v = leaf_value(name);
            let mut f = Field::new(fname, TypeRef::named("Json"), move |_| {
                let v = v.clone();
                FieldFuture::new(async move { Ok(Some(FieldValue::value(v))) })
            });
            for a in arg_inputs(&child.args) { f = f.argument(a); }
            obj = obj.field(f);
        } else {
            let child_ty = format!("{type_name}__{name}");
            build_object(&child_ty, child, objs);
            if nested_is_list(name) {
                let mut f = Field::new(fname, TypeRef::named_list(&child_ty), |_| {
                    FieldFuture::new(async { Ok(Some(FieldValue::list(Vec::<FieldValue>::new()))) })
                });
                for a in arg_inputs(&child.args) { f = f.argument(a); }
                obj = obj.field(f);
            } else {
                let mut f = Field::new(fname, TypeRef::named(&child_ty), |_| {
                    FieldFuture::new(async { Ok(Some(FieldValue::owned_any(()))) })
                });
                for a in arg_inputs(&child.args) { f = f.argument(a); }
                obj = obj.field(f);
            }
        }
    }
    // Objects must have at least one field.
    if node.children.is_empty() {
        obj = obj.field(Field::new("_", TypeRef::named("Json"), |_| {
            FieldFuture::new(async { Ok(Some(FieldValue::NULL)) })
        }));
    }
    objs.push(obj);
}

pub fn build() -> Result<Schema, SchemaError> {
    let mut roots: BTreeMap<(Kind, String), RootInfo> = BTreeMap::new();
    let mut scalars: BTreeSet<String> = BTreeSet::new();
    scalars.insert("Json".into());

    for op in operations() {
        let doc = async_graphql::parser::parse_query(&op.text).expect("UI operation parses");
        let var_types: BTreeMap<String, String> =
            op.vars.iter().map(|v| (v.name.clone(), v.ty.clone())).collect();
        for t in var_types.values() {
            if let Ok(parsed) = GType::new(t).ok_or(()) {
                named_types(&parsed, &mut scalars);
            }
        }
        for (_, operation) in doc.operations.iter() {
            let kind = match operation.node.ty {
                OperationType::Query => Kind::Query,
                OperationType::Mutation => Kind::Mutation,
                OperationType::Subscription => Kind::Subscription,
            };
            for sel in &operation.node.selection_set.node.items {
                if let Selection::Field(f) = &sel.node {
                    let field = f.node.name.node.to_string();
                    if field.starts_with("__") {
                        continue;
                    }
                    let info = roots.entry((kind, field)).or_default();
                    record_args(f.node.arguments.iter().map(|(n, v)| (n.node.to_string(), v.node.to_string())), &var_types, &mut info.tree.args);
                    collect(&f.node.selection_set.node, &doc, &var_types, &mut info.tree);
                }
            }
        }
    }

    // Argument types declared inline as literals fall back to Json.
    for b in BUILTIN {
        scalars.remove(b);
    }

    let mut objs = Vec::new();
    let mut query = Object::new("query_root");
    let mut mutation = Object::new("mutation_root");
    let mut subscription = Subscription::new("subscription_root");

    for ((kind, field), info) in &roots {
        let ty_name = format!("{}__{}", match kind { Kind::Query => "q", Kind::Mutation => "m", Kind::Subscription => "s" }, field);
        let scalar_root = info.tree.children.is_empty();
        if !scalar_root {
            build_object(&ty_name, &info.tree, &mut objs);
        }
        let single = root_is_single(field) || is_mutation_response(field);
        let ret = if scalar_root { TypeRef::named("Json") } else if single { TypeRef::named(&ty_name) } else { TypeRef::named_list(&ty_name) };
        let aggregate_like = field.ends_with("_aggregate") || is_mutation_response(field);
        let resolve_obj = move || -> Option<FieldValue<'static>> {
            if scalar_root {
                None
            } else if !single {
                Some(FieldValue::list(Vec::<FieldValue>::new()))
            } else if aggregate_like {
                Some(FieldValue::owned_any(()))
            } else {
                None
            }
        };
        let args = arg_inputs(&info.tree.args);
        match kind {
            Kind::Query | Kind::Mutation => {
                let mut f = Field::new(field.clone(), ret, move |_| {
                    let v = resolve_obj();
                    FieldFuture::new(async move { Ok(v) })
                });
                for a in args { f = f.argument(a); }
                if *kind == Kind::Query { query = query.field(f) } else { mutation = mutation.field(f) }
            }
            Kind::Subscription => {
                let mut f = SubscriptionField::new(field.clone(), ret, move |_| {
                    SubscriptionFieldFuture::new(async move {
                        let v = resolve_obj().unwrap_or(FieldValue::NULL);
                        Ok(futures_util::stream::once(async move { Ok(v) }))
                    })
                });
                for a in args { f = f.argument(a); }
                subscription = subscription.field(f);
            }
        }
    }

    let mut b = Schema::build("query_root", Some("mutation_root"), Some("subscription_root"))
        .register(query)
        .register(mutation)
        .register(subscription);
    for s in scalars {
        b = b.register(Scalar::new(s));
    }
    for o in objs {
        b = b.register(o);
    }
    let _ = Name::new("x");
    b.finish()
}
