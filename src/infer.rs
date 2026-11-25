use crate::{
    engine::Engine,
    types::{Fact, Length},
};

impl Engine {
    pub fn infer_facts(&mut self) {
        loop {
            let mut new_facts = Vec::new();
            let mut on_circle = Vec::new();
            let mut centers = Vec::new();
            let mut equal_lengths = Vec::new();
            for fact in &self.facts {
                match fact.normalize() {
                    Fact::CenterOfCircle(center, circle) => {
                        new_facts.push(Fact::InsideCircle(center, circle));
                        centers.push((center, circle));
                    }
                    Fact::OnCircle(point, circle) => on_circle.push((point, circle)),
                    Fact::EqualLengths(a, b) => equal_lengths.push((a, b)),
                    _ => {}
                }
            }

            for &(center, circle) in &centers {
                for &(point_p, circle_p) in &on_circle {
                    for &(point_q, circle_q) in &on_circle {
                        if circle_p == circle && circle_q == circle {
                            let center_to_p = Length::new(center, point_p);
                            let center_to_q = Length::new(center, point_q);
                            if center_to_p != center_to_q {
                                new_facts.push(Fact::EqualLengths(center_to_p, center_to_q));
                            }
                        }
                    }
                }
            }

            for &(a, b) in &equal_lengths {
                for &(c, d) in &equal_lengths {
                    if a == c && b != d {
                        new_facts.push(Fact::EqualLengths(b, d));
                    }
                    if a == d && b != c {
                        new_facts.push(Fact::EqualLengths(b, c));
                    }
                    if b == c && a != d {
                        new_facts.push(Fact::EqualLengths(a, d));
                    }
                    if b == d && a != c {
                        new_facts.push(Fact::EqualLengths(a, c));
                    }
                }
            }

            for &(point_p, circle_alpha) in &on_circle {
                for &(point_q, circle_beta) in &on_circle {
                    if self.contains(&Fact::InsideCircle(point_q, circle_alpha))
                        && self.contains(&Fact::InsideCircle(point_p, circle_beta))
                    {
                        new_facts.push(Fact::CirclesIntersect(circle_alpha, circle_beta));
                    }
                }
            }

            let mut changed = false;
            for fact in new_facts {
                changed |= self.insert(fact);
            }
            if !changed {
                return;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        engine::Engine,
        types::{CircleId, Fact, Length, PointId, Sorts},
    };

    #[test]
    fn infers_i1_equalities_and_saturates() {
        let a = PointId(1);
        let b = PointId(2);
        let c = PointId(3);
        let alpha = CircleId(1);
        let beta = CircleId(2);
        let mut state = Engine::default();
        state.known_objects.extend([
            Sorts::Point(a),
            Sorts::Point(b),
            Sorts::Point(c),
            Sorts::Circle(alpha),
            Sorts::Circle(beta),
        ]);
        for fact in [
            Fact::CenterOfCircle(a, alpha),
            Fact::OnCircle(b, alpha),
            Fact::OnCircle(c, alpha),
            Fact::CenterOfCircle(b, beta),
            Fact::OnCircle(a, beta),
            Fact::OnCircle(c, beta),
        ] {
            state.insert(fact);
        }

        state.infer_facts();
        assert!(state.contains(&Fact::CirclesIntersect(alpha, beta)));
        assert!(state.contains(&Fact::EqualLengths(Length::new(a, b), Length::new(b, c),)));
        assert!(state.contains(&Fact::EqualLengths(Length::new(b, c), Length::new(c, a),)));

        let stable_facts = state.facts.clone();
        state.infer_facts();
        assert_eq!(state.facts, stable_facts);
    }

    #[test]
    fn unrelated_circles_do_not_imply_equal_lengths_or_intersection() {
        let a = PointId(1);
        let b = PointId(2);
        let c = PointId(3);
        let d = PointId(4);
        let e = PointId(5);
        let f = PointId(6);
        let alpha = CircleId(1);
        let beta = CircleId(2);
        let mut state = Engine::default();
        state.known_objects.extend([
            Sorts::Point(a),
            Sorts::Point(b),
            Sorts::Point(c),
            Sorts::Point(d),
            Sorts::Point(e),
            Sorts::Point(f),
            Sorts::Circle(alpha),
            Sorts::Circle(beta),
        ]);
        for fact in [
            Fact::CenterOfCircle(a, alpha),
            Fact::OnCircle(b, alpha),
            Fact::OnCircle(c, alpha),
            Fact::CenterOfCircle(d, beta),
            Fact::OnCircle(e, beta),
            Fact::OnCircle(f, beta),
        ] {
            state.insert(fact);
        }

        state.infer_facts();
        assert!(state.contains(&Fact::EqualLengths(Length::new(a, b), Length::new(a, c),)));
        assert!(state.contains(&Fact::EqualLengths(Length::new(d, e), Length::new(d, f),)));
        assert!(!state.contains(&Fact::CirclesIntersect(alpha, beta)));
        assert!(!state.contains(&Fact::EqualLengths(Length::new(a, b), Length::new(d, e),)));
    }

    #[test]
    fn equality_transitivity_handles_each_shared_position() {
        let p1 = PointId(1);
        let p2 = PointId(2);
        let p3 = PointId(3);
        let p4 = PointId(4);
        let p5 = PointId(5);
        let lengths = [
            Length::new(p1, p2),
            Length::new(p1, p3),
            Length::new(p1, p4),
            Length::new(p1, p5),
        ];
        let cases = [
            (
                (lengths[0], lengths[1]),
                (lengths[0], lengths[2]),
                (lengths[1], lengths[2]),
            ),
            (
                (lengths[1], lengths[3]),
                (lengths[0], lengths[1]),
                (lengths[0], lengths[3]),
            ),
            (
                (lengths[0], lengths[1]),
                (lengths[1], lengths[3]),
                (lengths[0], lengths[3]),
            ),
            (
                (lengths[0], lengths[2]),
                (lengths[1], lengths[2]),
                (lengths[0], lengths[1]),
            ),
        ];

        for ((a, b), (c, d), (expected_a, expected_b)) in cases {
            let mut state = Engine::default();
            state.known_objects.extend([
                Sorts::Point(p1),
                Sorts::Point(p2),
                Sorts::Point(p3),
                Sorts::Point(p4),
                Sorts::Point(p5),
            ]);
            state.insert(Fact::EqualLengths(a, b));
            state.insert(Fact::EqualLengths(c, d));

            state.infer_facts();
            assert!(state.contains(&Fact::EqualLengths(expected_a, expected_b)));
        }
    }
}
