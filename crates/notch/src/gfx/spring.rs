pub struct Spring {
    pub position: f32,
    pub target: f32,
    velocity: f32,
    stiffness: f32,
    damping: f32,
    /// Seconds over which the pull toward a new target fades in from nothing, so a move starts gently.
    lead_in_seconds: f32,
    lead_in_elapsed: f32,
    last_target: f32,
}

const SUBSTEP_SECONDS: f32 = 1.0 / 240.0;
const SETTLE_DISTANCE: f32 = 0.0005;
const SETTLE_VELOCITY: f32 = 0.005;
/// A spring slower than this counts as resting when it gets a new target.
const LEAD_IN_REST_VELOCITY: f32 = 0.05;

impl Spring {
    pub fn new(position: f32, stiffness: f32, damping: f32) -> Self {
        Self {
            position,
            target: position,
            velocity: 0.0,
            stiffness,
            damping,
            lead_in_seconds: 0.0,
            lead_in_elapsed: 0.0,
            last_target: position,
        }
    }

    /// Eases the start of every move that begins at rest: the pull toward the target grows along a smoothstep curve.
    pub fn with_lead_in(mut self, seconds: f32) -> Self {
        self.lead_in_seconds = seconds;
        self
    }

    /// Share of the full pull at this moment; a retarget while moving keeps the pull it had.
    fn pull_gain(&self) -> f32 {
        if self.lead_in_elapsed >= self.lead_in_seconds {
            return 1.0;
        }
        let progress = self.lead_in_elapsed / self.lead_in_seconds;
        progress * progress * (3.0 - 2.0 * progress)
    }

    pub fn step(&mut self, elapsed_seconds: f32) {
        if self.target != self.last_target {
            self.last_target = self.target;
            if self.velocity.abs() < LEAD_IN_REST_VELOCITY {
                self.lead_in_elapsed = 0.0;
            }
        }
        let mut remaining = elapsed_seconds;
        while remaining > 0.0 {
            let substep = remaining.min(SUBSTEP_SECONDS);
            let pull = self.stiffness * self.pull_gain() * (self.target - self.position);
            let acceleration = pull - self.damping * self.velocity;
            self.lead_in_elapsed += substep;
            self.velocity += acceleration * substep;
            self.position += self.velocity * substep;
            remaining -= substep;
        }
    }

    pub fn is_settled(&self) -> bool {
        (self.target - self.position).abs() < SETTLE_DISTANCE && self.velocity.abs() < SETTLE_VELOCITY
    }

    pub fn jump_to(&mut self, value: f32) {
        self.position = value;
        self.target = value;
        self.velocity = 0.0;
    }

    pub fn snap_to_target(&mut self) {
        self.position = self.target;
        self.velocity = 0.0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn run(spring: &mut Spring, seconds: f32) {
        let frame = 1.0 / 60.0;
        for _ in 0..(seconds / frame) as usize {
            spring.step(frame);
        }
    }

    #[test]
    fn a_new_spring_is_settled_at_its_start() {
        assert!(Spring::new(3.0, 200.0, 26.0).is_settled());
    }

    #[test]
    fn settles_on_its_target() {
        let mut spring = Spring::new(0.0, 220.0, 26.0);
        spring.target = 1.0;
        assert!(!spring.is_settled());
        run(&mut spring, 3.0);
        assert!(spring.is_settled());
        assert!((spring.position - 1.0).abs() < 0.001);
    }

    #[test]
    fn a_lightly_damped_spring_overshoots() {
        let mut spring = Spring::new(0.0, 260.0, 22.0);
        spring.target = 1.0;
        let mut highest = 0.0_f32;
        for _ in 0..120 {
            spring.step(1.0 / 60.0);
            highest = highest.max(spring.position);
        }
        assert!(highest > 1.0, "expected overshoot, peaked at {highest}");
    }

    #[test]
    fn a_lead_in_starts_a_move_slower_than_a_plain_spring_and_still_arrives() {
        let (mut plain, mut eased) = (Spring::new(0.0, 200.0, 16.0), Spring::new(0.0, 200.0, 16.0).with_lead_in(0.1));
        (plain.target, eased.target) = (1.0, 1.0);
        for _ in 0..6 {
            plain.step(1.0 / 60.0);
            eased.step(1.0 / 60.0);
        }
        assert!(eased.position < plain.position * 0.5, "{} vs {}", eased.position, plain.position);
        run(&mut eased, 3.0);
        assert!(eased.is_settled());
    }

    #[test]
    fn a_retarget_while_moving_does_not_restart_the_lead_in() {
        let mut spring = Spring::new(0.0, 200.0, 16.0).with_lead_in(0.1);
        spring.target = 1.0;
        run(&mut spring, 0.5);
        spring.target = 0.0;
        spring.step(1.0 / 60.0);
        assert_eq!(spring.pull_gain(), 1.0);
    }

    #[test]
    fn jump_to_moves_without_motion() {
        let mut spring = Spring::new(0.0, 220.0, 26.0);
        spring.target = 5.0;
        spring.step(0.05);
        spring.jump_to(2.0);
        assert!(spring.is_settled());
        assert_eq!((spring.position, spring.target), (2.0, 2.0));
    }

    #[test]
    fn snap_to_target_finishes_the_move() {
        let mut spring = Spring::new(0.0, 220.0, 26.0);
        spring.target = 1.0;
        spring.step(0.01);
        spring.snap_to_target();
        assert!(spring.is_settled());
    }
}
