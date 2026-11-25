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
            let mut inside_facts = Vec::new();
            let mut equal_lengths = Vec::new();
            for fact in &self.facts {
                match fact.normalize() {
                    Fact::CenterOfCircle(center, circle) => {
                        new_facts.push(Fact::InsideCircle(center, circle));
                        centers.push((center, circle));
                    }
                    Fact::OnCircle(point, circle) => on_circle.push((point, circle)),
                    Fact::InsideCircle(point, circle) => inside_facts.push((point, circle)),
                    Fact::EqualLengths(a, b) => equal_lengths.push((a, b)),
                    _ => {}
                }
            }

            for &(inside_point, inside_gamma) in &inside_facts {
                for &(boundary_point, boundary_circle) in &on_circle {
                    if inside_gamma == boundary_circle && inside_point != boundary_point {
                        new_facts.push(Fact::DistinctPoints(inside_point, boundary_point));
                    }
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
