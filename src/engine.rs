use std::collections::HashSet;

use crate::types::{CircleId, Fact, PointId, Sorts};

#[derive(Debug, PartialEq, Eq)]
pub enum ConstructionError {
    UnknownPoint(PointId),
    UnknownCircle(CircleId),
    PointsNotKnownDistinct(PointId, PointId),
    CirclesNotKnownIntersecting(CircleId, CircleId),
    IdentifierSpaceExhausted,
}

#[derive(Default)]
pub struct Engine {
    pub facts: HashSet<Fact>,
    pub known_objects: HashSet<Sorts>,
    pub assumptions: HashSet<Fact>,
    next_point_id: u64,
    next_circle_id: u64,
}

impl Engine {
    pub fn insert(&mut self, fact: Fact) -> bool {
        let fact = fact.normalize();
        self.facts.insert(fact)
    }

    pub fn contains(&self, fact: &Fact) -> bool {
        let fact = fact.normalize();
        self.facts.contains(&fact)
    }

    pub fn given_point(&mut self) -> Result<PointId, ConstructionError> {
        let point = self.fresh_point_id()?;
        self.register_point(point);
        Ok(point)
    }

    pub fn assume_distinct_points(
        &mut self,
        a: PointId,
        b: PointId,
    ) -> Result<(), ConstructionError> {
        for point in [a, b] {
            if !self.known_objects.contains(&Sorts::Point(point)) {
                return Err(ConstructionError::UnknownPoint(point));
            }
        }

        let fact = Fact::DistinctPoints(a, b).normalize();
        self.assumptions.insert(fact);
        self.insert(fact);
        Ok(())
    }

    pub fn construct_circle(
        &mut self,
        center: PointId,
        through: PointId,
    ) -> Result<CircleId, ConstructionError> {
        for point in [center, through] {
            if !self.known_objects.contains(&Sorts::Point(point)) {
                return Err(ConstructionError::UnknownPoint(point));
            }
        }
        if !self.contains(&Fact::DistinctPoints(center, through)) {
            return Err(ConstructionError::PointsNotKnownDistinct(center, through));
        }

        let circle = self.fresh_circle_id()?;
        self.register_circle(circle);
        self.insert(Fact::CenterOfCircle(center, circle));
        self.insert(Fact::OnCircle(through, circle));
        self.infer_facts();
        Ok(circle)
    }

    pub fn construct_intersection(
        &mut self,
        alpha: CircleId,
        beta: CircleId,
    ) -> Result<PointId, ConstructionError> {
        for circle in [alpha, beta] {
            if !self.known_objects.contains(&Sorts::Circle(circle)) {
                return Err(ConstructionError::UnknownCircle(circle));
            }
        }
        if !self.contains(&Fact::CirclesIntersect(alpha, beta)) {
            return Err(ConstructionError::CirclesNotKnownIntersecting(alpha, beta));
        }

        let point = self.fresh_point_id()?;
        self.register_point(point);
        self.insert(Fact::OnCircle(point, alpha));
        self.insert(Fact::OnCircle(point, beta));
        self.infer_facts();
        Ok(point)
    }

    fn fresh_point_id(&self) -> Result<PointId, ConstructionError> {
        let mut next = self.next_point_id;
        loop {
            let raw =
                u32::try_from(next).map_err(|_| ConstructionError::IdentifierSpaceExhausted)?;
            let point = PointId(raw);
            if !self.known_objects.contains(&Sorts::Point(point)) {
                return Ok(point);
            }
            next = next
                .checked_add(1)
                .ok_or(ConstructionError::IdentifierSpaceExhausted)?;
        }
    }

    fn register_point(&mut self, point: PointId) {
        self.known_objects.insert(Sorts::Point(point));
        self.next_point_id = u64::from(point.0) + 1;
    }

    fn fresh_circle_id(&self) -> Result<CircleId, ConstructionError> {
        let mut next = self.next_circle_id;
        loop {
            let raw =
                u32::try_from(next).map_err(|_| ConstructionError::IdentifierSpaceExhausted)?;
            let circle = CircleId(raw);
            if !self.known_objects.contains(&Sorts::Circle(circle)) {
                return Ok(circle);
            }
            next = next
                .checked_add(1)
                .ok_or(ConstructionError::IdentifierSpaceExhausted)?;
        }
    }

    fn register_circle(&mut self, circle: CircleId) {
        self.known_objects.insert(Sorts::Circle(circle));
        self.next_circle_id = u64::from(circle.0) + 1;
    }
}
