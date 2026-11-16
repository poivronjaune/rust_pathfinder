use std::collections::{HashMap, VecDeque};

use crate::simulator::{Command, Direction, Outcome};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub(crate) struct Position {
    pub(crate) row: i32,
    pub(crate) col: i32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Terrain {
    Unknown,
    Free,
    Obstacle,
    Boundary,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ExecutionState {
    Idle,
    Running,
    Paused,
    Inspecting,
    Completed,
    Blocked,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Event {
    RequestInspection(Position),
    WaitForInspection,
    Drive(Command),
    MissionCompleted,
    UnableToProgress,
}

#[derive(Clone, Debug)]
pub(crate) struct Navigator {
    terrain: HashMap<Position, Terrain>,
    actions: HashMap<Position, bool>,
    position: Position,
    heading: Direction,
    travel: Vec<Position>,
    action_enabled: bool,
    state: ExecutionState,
    pending: Option<Position>,
}

impl Navigator {
    pub(crate) fn new(action_enabled: bool) -> Self {
        let mut navigator = Self {
            terrain: HashMap::new(),
            actions: HashMap::new(),
            position: Position { row: 0, col: 0 },
            heading: Direction::Up,
            travel: Vec::new(),
            action_enabled,
            state: ExecutionState::Idle,
            pending: None,
        };
        navigator.reset_map();
        navigator
    }

    pub(crate) fn state(&self) -> ExecutionState {
        self.state
    }

    pub(crate) fn position(&self) -> Position {
        self.position
    }

    pub(crate) fn heading(&self) -> Direction {
        self.heading
    }

    pub(crate) fn travel(&self) -> &[Position] {
        &self.travel
    }

    pub(crate) fn terrain(&self) -> &HashMap<Position, Terrain> {
        &self.terrain
    }

    pub(crate) fn actions(&self) -> &HashMap<Position, bool> {
        &self.actions
    }

    pub(crate) fn pending_inspection(&self) -> Option<Position> {
        self.pending
    }

    pub(crate) fn start(&mut self) {
        if matches!(
            self.state,
            ExecutionState::Idle | ExecutionState::Paused | ExecutionState::Inspecting
        ) {
            self.state = ExecutionState::Running;
        }
    }

    pub(crate) fn pause(&mut self) {
        if matches!(
            self.state,
            ExecutionState::Running | ExecutionState::Inspecting
        ) {
            self.state = ExecutionState::Paused;
        }
    }

    pub(crate) fn step(&mut self) -> Event {
        if self.state == ExecutionState::Completed {
            return Event::MissionCompleted;
        }
        if self.state == ExecutionState::Blocked {
            return Event::UnableToProgress;
        }
        let running = self.state == ExecutionState::Running;
        if self.pending.is_some() {
            self.state = ExecutionState::Inspecting;
            return Event::WaitForInspection;
        }

        if let Some(position) = self.first_forward_unknown() {
            self.pending = Some(position);
            self.state = ExecutionState::Inspecting;
            return Event::RequestInspection(position);
        }

        let forward = offset(self.position, self.heading, 1);
        if self.terrain.get(&forward) == Some(&Terrain::Free)
            && self.actions.get(&forward) == Some(&false)
        {
            self.state = if running {
                ExecutionState::Running
            } else {
                ExecutionState::Paused
            };
            return Event::Drive(Command::Forward);
        }

        if let Some((goal, direction)) = self.next_goal() {
            if self.position == goal {
                if self.heading != direction {
                    self.state = if running {
                        ExecutionState::Running
                    } else {
                        ExecutionState::Paused
                    };
                    return Event::Drive(turn_toward(self.heading, direction));
                }
                self.state = ExecutionState::Blocked;
                return Event::UnableToProgress;
            }
            if let Some(next) = self.next_route_step(goal) {
                let direction = direction_between(self.position, next);
                if self.heading != direction {
                    self.state = if running {
                        ExecutionState::Running
                    } else {
                        ExecutionState::Paused
                    };
                    return Event::Drive(turn_toward(self.heading, direction));
                }
                self.state = if running {
                    ExecutionState::Running
                } else {
                    ExecutionState::Paused
                };
                return Event::Drive(Command::Forward);
            }
            self.state = ExecutionState::Blocked;
            return Event::UnableToProgress;
        }

        self.state = ExecutionState::Completed;
        Event::MissionCompleted
    }

    pub(crate) fn classify(&mut self, position: Position, terrain: Terrain) -> bool {
        if self.pending != Some(position) || terrain == Terrain::Unknown {
            return false;
        }
        self.terrain.insert(position, terrain);
        if terrain == Terrain::Free {
            self.actions.entry(position).or_insert(false);
        }
        self.pending = None;
        self.state = ExecutionState::Paused;
        true
    }

    pub(crate) fn record_drive(
        &mut self,
        command: Command,
        outcome: Outcome,
        actual_position: Position,
        actual_heading: Direction,
    ) {
        self.position = actual_position;
        self.heading = actual_heading;
        match command {
            Command::Forward | Command::Reverse if outcome == Outcome::Moved => {
                self.travel.push(actual_position);
                if self.terrain.get(&actual_position) == Some(&Terrain::Free) && self.action_enabled
                {
                    self.actions.insert(actual_position, true);
                }
            }
            Command::Forward | Command::Reverse if outcome != Outcome::Moved => {
                let blocked = adjacent(self.position, self.heading, command == Command::Forward);
                if self.terrain.get(&blocked) == Some(&Terrain::Free) {
                    self.terrain.insert(blocked, Terrain::Unknown);
                    self.actions.remove(&blocked);
                    self.state = ExecutionState::Blocked;
                } else {
                    self.state = ExecutionState::Paused;
                }
            }
            _ => {}
        }
    }

    pub(crate) fn reset_session(&mut self, start: Position, heading: Direction) {
        self.position = start;
        self.heading = heading;
        self.travel.clear();
        self.travel.push(start);
        for completed in self.actions.values_mut() {
            *completed = false;
        }
        if self.terrain.get(&start) != Some(&Terrain::Free) {
            self.terrain.insert(start, Terrain::Free);
        }
        self.actions.insert(start, self.action_enabled);
        self.pending = None;
        self.state = ExecutionState::Idle;
    }

    pub(crate) fn reset_map(&mut self) {
        let start = Position { row: 0, col: 0 };
        self.terrain.clear();
        self.actions.clear();
        self.terrain.insert(start, Terrain::Free);
        self.position = start;
        self.heading = Direction::Up;
        self.travel.clear();
        self.travel.push(start);
        self.actions.insert(start, self.action_enabled);
        self.pending = None;
        self.state = ExecutionState::Idle;
    }

    pub(crate) fn synchronize(&mut self, position: Position, heading: Direction) {
        self.position = position;
        self.heading = heading;
        self.reset_session(position, heading);
    }

    fn first_forward_unknown(&self) -> Option<Position> {
        for distance in 1..=3 {
            let position = offset(self.position, self.heading, distance);
            match self
                .terrain
                .get(&position)
                .copied()
                .unwrap_or(Terrain::Unknown)
            {
                Terrain::Unknown => return Some(position),
                Terrain::Obstacle | Terrain::Boundary => return None,
                Terrain::Free => {}
            }
        }
        None
    }

    fn next_goal(&self) -> Option<(Position, Direction)> {
        let distances = self.reachable_distances();
        let mut goals = Vec::new();
        for (&position, &distance) in &distances {
            if self.actions.get(&position) == Some(&false) {
                goals.push((position, self.heading, distance));
            }
            for direction in directions() {
                if self.first_unknown_from(position, direction).is_some() {
                    goals.push((position, direction, distance));
                }
            }
        }
        goals
            .into_iter()
            .min_by_key(|(position, direction, distance)| {
                (
                    *distance
                        + usize::from(*position == self.position && *direction != self.heading),
                    *position,
                    direction_index(*direction),
                )
            })
            .map(|(position, direction, _)| (position, direction))
    }

    fn first_unknown_from(&self, position: Position, direction: Direction) -> Option<Position> {
        for distance in 1..=3 {
            let next = offset(position, direction, distance);
            match self.terrain.get(&next).copied().unwrap_or(Terrain::Unknown) {
                Terrain::Unknown => return Some(next),
                Terrain::Obstacle | Terrain::Boundary => return None,
                Terrain::Free => {}
            }
        }
        None
    }

    fn reachable_distances(&self) -> HashMap<Position, usize> {
        let mut distances = HashMap::from([(self.position, 0usize)]);
        let mut queue = VecDeque::from([self.position]);
        while let Some(position) = queue.pop_front() {
            for direction in directions() {
                let next = offset(position, direction, 1);
                if self.terrain.get(&next) == Some(&Terrain::Free) && !distances.contains_key(&next)
                {
                    distances.insert(next, distances[&position] + 1);
                    queue.push_back(next);
                }
            }
        }
        distances
    }

    fn next_route_step(&self, target: Position) -> Option<Position> {
        let mut queue = VecDeque::from([self.position]);
        let mut previous = HashMap::from([(self.position, self.position)]);
        while let Some(position) = queue.pop_front() {
            if position == target {
                break;
            }
            for direction in directions() {
                let next = offset(position, direction, 1);
                if self.terrain.get(&next) == Some(&Terrain::Free) && !previous.contains_key(&next)
                {
                    previous.insert(next, position);
                    queue.push_back(next);
                }
            }
        }
        if !previous.contains_key(&target) {
            return None;
        }
        let mut step = target;
        while previous[&step] != self.position {
            step = previous[&step];
        }
        Some(step)
    }
}

fn directions() -> [Direction; 4] {
    [
        Direction::Up,
        Direction::Right,
        Direction::Down,
        Direction::Left,
    ]
}

fn direction_index(direction: Direction) -> usize {
    directions()
        .iter()
        .position(|candidate| *candidate == direction)
        .unwrap_or(0)
}

fn offset(position: Position, direction: Direction, distance: i32) -> Position {
    let (row, col) = match direction {
        Direction::Up => (-distance, 0),
        Direction::Right => (0, distance),
        Direction::Down => (distance, 0),
        Direction::Left => (0, -distance),
    };
    Position {
        row: position.row + row,
        col: position.col + col,
    }
}

fn adjacent(position: Position, direction: Direction, forward: bool) -> Position {
    offset(position, direction, if forward { 1 } else { -1 })
}

fn direction_between(from: Position, to: Position) -> Direction {
    match (to.row - from.row, to.col - from.col) {
        (-1, 0) => Direction::Up,
        (0, 1) => Direction::Right,
        (1, 0) => Direction::Down,
        (0, -1) => Direction::Left,
        _ => unreachable!("route steps are adjacent"),
    }
}

fn turn_toward(current: Direction, target: Direction) -> Command {
    match (direction_index(target) + 4 - direction_index(current)) % 4 {
        1 => Command::TurnRight,
        2 => Command::TurnRight,
        3 => Command::TurnLeft,
        _ => unreachable!("turn is requested only when headings differ"),
    }
}

#[cfg(test)]
mod tests {
    use super::{Event, ExecutionState, Navigator, Position, Terrain};
    use crate::simulator::{Command, Direction, Outcome};

    #[test]
    fn only_start_is_initially_known_and_reset_map_clears_session() {
        let mut navigator = Navigator::new(true);
        assert_eq!(navigator.terrain().len(), 1);
        assert_eq!(
            navigator.terrain().get(&Position { row: 0, col: 0 }),
            Some(&Terrain::Free)
        );
        assert_eq!(navigator.travel(), &[Position { row: 0, col: 0 }]);
        navigator.classify(Position { row: -1, col: 0 }, Terrain::Obstacle);
        navigator.reset_map();
        assert_eq!(navigator.terrain().len(), 1);
    }

    #[test]
    fn observations_are_nearest_first_and_require_classification() {
        let mut navigator = Navigator::new(true);
        assert_eq!(
            navigator.step(),
            Event::RequestInspection(Position { row: -1, col: 0 })
        );
        assert_eq!(navigator.state(), ExecutionState::Inspecting);
        assert_eq!(navigator.step(), Event::WaitForInspection);
        assert!(navigator.classify(Position { row: -1, col: 0 }, Terrain::Free));
        assert!(!navigator.classify(Position { row: -2, col: 0 }, Terrain::Free));
        assert_eq!(
            navigator.step(),
            Event::RequestInspection(Position { row: -2, col: 0 })
        );
    }

    #[test]
    fn movement_records_only_success_and_performs_enabled_actions() {
        let mut navigator = Navigator::new(true);
        assert_eq!(
            navigator.step(),
            Event::RequestInspection(Position { row: -1, col: 0 })
        );
        navigator.classify(Position { row: -1, col: 0 }, Terrain::Free);
        navigator.record_drive(
            Command::Forward,
            Outcome::Moved,
            Position { row: -1, col: 0 },
            Direction::Up,
        );
        assert_eq!(
            navigator.travel(),
            &[Position { row: 0, col: 0 }, Position { row: -1, col: 0 }]
        );
        assert_eq!(
            navigator.actions().get(&Position { row: -1, col: 0 }),
            Some(&true)
        );

        navigator.record_drive(
            Command::Forward,
            Outcome::AtEdge,
            Position { row: -1, col: 0 },
            Direction::Up,
        );
        assert_eq!(navigator.travel().len(), 2);
    }

    #[test]
    fn reset_session_preserves_terrain_and_clears_action_and_travel_state() {
        let mut navigator = Navigator::new(true);
        let next = Position { row: -1, col: 0 };
        navigator.step();
        navigator.classify(next, Terrain::Free);
        navigator.record_drive(Command::Forward, Outcome::Moved, next, Direction::Up);
        navigator.reset_session(Position { row: 0, col: 0 }, Direction::Up);
        assert_eq!(navigator.terrain().get(&next), Some(&Terrain::Free));
        assert_eq!(navigator.travel(), &[Position { row: 0, col: 0 }]);
        assert_eq!(navigator.actions().get(&next), Some(&false));
    }

    #[test]
    fn unreachable_frontiers_are_sought_by_turning_and_known_free_routes() {
        let mut navigator = Navigator::new(true);
        navigator.step();
        navigator.classify(Position { row: -1, col: 0 }, Terrain::Obstacle);
        assert_eq!(navigator.step(), Event::Drive(Command::TurnRight));
    }

    #[test]
    fn completed_requires_exhausting_each_reachable_observation_frontier() {
        let mut navigator = Navigator::new(true);
        let origin = Position { row: 0, col: 0 };
        let walls = [
            (Position { row: -1, col: 0 }, Direction::Up),
            (Position { row: 0, col: 1 }, Direction::Right),
            (Position { row: 1, col: 0 }, Direction::Down),
            (Position { row: 0, col: -1 }, Direction::Left),
        ];

        assert_eq!(navigator.step(), Event::RequestInspection(walls[0].0));
        for (index, (position, heading)) in walls.into_iter().enumerate() {
            assert!(navigator.classify(position, Terrain::Obstacle));
            if index < 3 {
                assert_eq!(navigator.step(), Event::Drive(Command::TurnRight));
                let next_heading = walls[index + 1].1;
                navigator.record_drive(Command::TurnRight, Outcome::Turned, origin, next_heading);
                assert_eq!(
                    navigator.step(),
                    Event::RequestInspection(walls[index + 1].0)
                );
            } else {
                assert_eq!(navigator.step(), Event::MissionCompleted);
            }
            let _ = heading;
        }
        assert_eq!(navigator.heading(), Direction::Left);
        assert_eq!(navigator.state(), ExecutionState::Completed);
    }

    #[test]
    fn disabled_actions_remain_incomplete_after_successful_entry() {
        let mut navigator = Navigator::new(false);
        assert_eq!(
            navigator.step(),
            Event::RequestInspection(Position { row: -1, col: 0 })
        );
        let next = Position { row: -1, col: 0 };
        navigator.classify(next, Terrain::Free);
        navigator.record_drive(Command::Forward, Outcome::Moved, next, Direction::Up);
        assert_eq!(navigator.actions().get(&next), Some(&false));
    }
}
