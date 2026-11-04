use merlin_framework::Duration;
use std::collections::{BTreeMap, HashMap};

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ActivityId(pub u64);

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct ActivityType(pub String);

impl From<&str> for ActivityType {
    fn from(value: &str) -> Self {
        Self(value.to_owned())
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Activity {
    pub id: ActivityId,
    pub activity_type: ActivityType,
    pub start: Duration,
    pub duration: Duration,
    pub arguments: BTreeMap<String, Vec<u8>>,
    pub parent: Option<ActivityId>,
    pub anchor: Option<ActivityId>,
    pub anchored_to_start: bool,
    pub name: Option<String>,
}

impl Activity {
    pub fn end(&self) -> Option<Duration> {
        self.start.checked_add(self.duration)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PlanningHorizon {
    pub start: Duration,
    pub end: Duration,
}

impl PlanningHorizon {
    pub fn new(start: Duration, end: Duration) -> Result<Self, ScheduleError> {
        if start > end {
            return Err(ScheduleError::InvalidHorizon);
        }
        Ok(Self { start, end })
    }

    pub fn contains(&self, time: Duration) -> bool {
        self.start <= time && time < self.end
    }

    pub fn duration(&self) -> Duration {
        self.end - self.start
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ScheduleError {
    InvalidHorizon,
    NegativeDuration(ActivityId),
    TimeOverflow(ActivityId),
    OutsideHorizon(ActivityId),
    DuplicateActivity(ActivityId),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Plan {
    horizon: PlanningHorizon,
    activities: HashMap<ActivityId, Activity>,
}

impl Plan {
    pub fn new(horizon: PlanningHorizon) -> Self {
        Self {
            horizon,
            activities: HashMap::new(),
        }
    }

    pub fn horizon(&self) -> PlanningHorizon {
        self.horizon
    }

    pub fn insert(&mut self, activity: Activity) -> Result<(), ScheduleError> {
        if self.activities.contains_key(&activity.id) {
            return Err(ScheduleError::DuplicateActivity(activity.id));
        }
        if activity.duration < Duration::ZERO {
            return Err(ScheduleError::NegativeDuration(activity.id));
        }

        let Some(end) = activity.end() else {
            return Err(ScheduleError::TimeOverflow(activity.id));
        };
        if !self.horizon.contains(activity.start) || end > self.horizon.end {
            return Err(ScheduleError::OutsideHorizon(activity.id));
        }

        self.activities.insert(activity.id, activity);
        Ok(())
    }

    pub fn get(&self, id: ActivityId) -> Option<&Activity> {
        self.activities.get(&id)
    }

    pub fn len(&self) -> usize {
        self.activities.len()
    }

    pub fn is_empty(&self) -> bool {
        self.activities.is_empty()
    }

    pub fn activities(&self) -> impl Iterator<Item = &Activity> {
        self.activities.values()
    }
}

#[cfg(test)]
mod tests {
    use super::{Activity, ActivityId, ActivityType, Plan, PlanningHorizon, ScheduleError};
    use merlin_framework::Duration;
    use std::collections::BTreeMap;

    fn activity(id: u64, start: i64, duration: i64) -> Activity {
        Activity {
            id: ActivityId(id),
            activity_type: ActivityType::from("observe"),
            start: Duration::microseconds(start),
            duration: Duration::microseconds(duration),
            arguments: BTreeMap::new(),
            parent: None,
            anchor: None,
            anchored_to_start: true,
            name: None,
        }
    }

    fn plan() -> Plan {
        Plan::new(
            PlanningHorizon::new(Duration::microseconds(0), Duration::microseconds(10)).unwrap(),
        )
    }

    #[test]
    fn horizon_is_half_open() {
        let horizon =
            PlanningHorizon::new(Duration::microseconds(2), Duration::microseconds(8)).unwrap();
        assert!(!horizon.contains(Duration::microseconds(1)));
        assert!(horizon.contains(Duration::microseconds(2)));
        assert!(!horizon.contains(Duration::microseconds(8)));
        assert_eq!(horizon.duration(), Duration::microseconds(6));
    }

    #[test]
    fn insert_rejects_duplicate_and_out_of_horizon_activities() {
        let mut plan = plan();
        plan.insert(activity(1, 2, 3)).unwrap();
        assert_eq!(plan.len(), 1);
        assert_eq!(
            plan.insert(activity(1, 4, 1)),
            Err(ScheduleError::DuplicateActivity(ActivityId(1)))
        );
        assert_eq!(
            plan.insert(activity(2, 9, 2)),
            Err(ScheduleError::OutsideHorizon(ActivityId(2)))
        );
        assert_eq!(plan.len(), 1);
    }

    #[test]
    fn insert_rejects_negative_duration_without_mutating_plan() {
        let mut plan = plan();
        assert_eq!(
            plan.insert(activity(3, 4, -1)),
            Err(ScheduleError::NegativeDuration(ActivityId(3)))
        );
        assert!(plan.is_empty());
    }
}
