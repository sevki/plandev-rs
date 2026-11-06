use crate::Duration;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum InSpan {
    #[default]
    Parent,
    Fresh,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TaskAction {
    Delay(Duration),
    Spawn { task: String, span: InSpan },
    Call { task: String, span: InSpan },
    WaitUntil(String),
    Emit { topic: String, value: Vec<u8> },
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct ModelActions {
    actions: Vec<TaskAction>,
}

impl ModelActions {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn delay(&mut self, duration: Duration) {
        self.actions.push(TaskAction::Delay(duration));
    }

    pub fn spawn(&mut self, task: impl Into<String>) {
        self.spawn_with_span(task, InSpan::Parent);
    }

    pub fn spawn_with_span(&mut self, task: impl Into<String>, span: InSpan) {
        self.actions.push(TaskAction::Spawn {
            task: task.into(),
            span,
        });
    }

    pub fn call(&mut self, task: impl Into<String>) {
        self.call_with_span(task, InSpan::Parent);
    }

    pub fn call_with_span(&mut self, task: impl Into<String>, span: InSpan) {
        self.actions.push(TaskAction::Call {
            task: task.into(),
            span,
        });
    }

    pub fn defer(&mut self, duration: Duration, task: impl Into<String>, span: InSpan) {
        self.delay(duration);
        self.spawn_with_span(task, span);
    }

    pub fn wait_until(&mut self, condition: impl Into<String>) {
        self.actions.push(TaskAction::WaitUntil(condition.into()));
    }

    pub fn emit(&mut self, topic: impl Into<String>, value: impl Into<Vec<u8>>) {
        self.actions.push(TaskAction::Emit {
            topic: topic.into(),
            value: value.into(),
        });
    }

    pub fn actions(&self) -> &[TaskAction] {
        &self.actions
    }

    pub fn into_actions(self) -> Vec<TaskAction> {
        self.actions
    }
}

#[cfg(test)]
mod tests {
    use super::{InSpan, ModelActions, TaskAction};
    use crate::Duration;

    #[test]
    fn deferral_waits_then_spawns_with_requested_span() {
        let mut actions = ModelActions::new();
        actions.defer(Duration::microseconds(5), "observe", InSpan::Fresh);

        assert_eq!(
            actions.into_actions(),
            vec![
                TaskAction::Delay(Duration::microseconds(5)),
                TaskAction::Spawn {
                    task: "observe".into(),
                    span: InSpan::Fresh,
                },
            ]
        );
    }

    #[test]
    fn task_operations_default_to_parent_span() {
        let mut actions = ModelActions::new();
        actions.spawn("child");
        actions.call("subtask");

        assert_eq!(
            actions.actions(),
            &[
                TaskAction::Spawn {
                    task: "child".into(),
                    span: InSpan::Parent,
                },
                TaskAction::Call {
                    task: "subtask".into(),
                    span: InSpan::Parent,
                },
            ]
        );
    }
}
