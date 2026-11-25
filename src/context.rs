use std::collections::HashMap;

use crate::{
    engine::{ConstructionError, Engine},
    types::{CircleId, Fact, Length, PointId, Sorts},
};

#[derive(Debug, PartialEq, Eq)]
pub enum ContextError {
    DuplicateName(String),
    UnknownName(String),
    ExpectedPoint(String),
    ExpectedCircle(String),
    Construction(ConstructionError),
}

#[derive(Default)]
pub struct Context {
    pub engine: Engine,
    pub names: HashMap<String, Sorts>,
}

impl Context {
    pub fn given_point(&mut self, name: impl Into<String>) -> Result<PointId, ContextError> {
        let name = name.into();
        self.ensure_name_available(&name)?;
        let point = self
            .engine
            .given_point()
            .map_err(ContextError::Construction)?;
        self.names.insert(name, Sorts::Point(point));
        Ok(point)
    }

    pub fn assume_distinct_points(
        &mut self,
        a_name: &str,
        b_name: &str,
    ) -> Result<(), ContextError> {
        let a = self.resolve_point(a_name)?;
        let b = self.resolve_point(b_name)?;
        self.engine
            .assume_distinct_points(a, b)
            .map_err(ContextError::Construction)
    }

    pub fn construct_circle(
        &mut self,
        name: impl Into<String>,
        center_name: &str,
        through_name: &str,
    ) -> Result<CircleId, ContextError> {
        let name = name.into();
        self.ensure_name_available(&name)?;
        let center = self.resolve_point(center_name)?;
        let through = self.resolve_point(through_name)?;
        let circle = self
            .engine
            .construct_circle(center, through)
            .map_err(ContextError::Construction)?;
        self.names.insert(name, Sorts::Circle(circle));
        Ok(circle)
    }

    pub fn construct_intersection(
        &mut self,
        name: impl Into<String>,
        alpha_name: &str,
        beta_name: &str,
    ) -> Result<PointId, ContextError> {
        let name = name.into();
        self.ensure_name_available(&name)?;
        let alpha = self.resolve_circle(alpha_name)?;
        let beta = self.resolve_circle(beta_name)?;
        let point = self
            .engine
            .construct_intersection(alpha, beta)
            .map_err(ContextError::Construction)?;
        self.names.insert(name, Sorts::Point(point));
        Ok(point)
    }

    pub fn has_equal_lengths(
        &self,
        a_name: &str,
        b_name: &str,
        c_name: &str,
        d_name: &str,
    ) -> Result<bool, ContextError> {
        let a = self.resolve_point(a_name)?;
        let b = self.resolve_point(b_name)?;
        let c = self.resolve_point(c_name)?;
        let d = self.resolve_point(d_name)?;
        Ok(self
            .engine
            .contains(&Fact::EqualLengths(Length::new(a, b), Length::new(c, d))))
    }

    fn ensure_name_available(&self, name: &str) -> Result<(), ContextError> {
        if self.names.contains_key(name) {
            return Err(ContextError::DuplicateName(name.to_owned()));
        }
        Ok(())
    }

    fn resolve_point(&self, name: &str) -> Result<PointId, ContextError> {
        match self.names.get(name) {
            Some(Sorts::Point(point)) => Ok(*point),
            Some(Sorts::Circle(_)) => Err(ContextError::ExpectedPoint(name.to_owned())),
            None => Err(ContextError::UnknownName(name.to_owned())),
        }
    }

    fn resolve_circle(&self, name: &str) -> Result<CircleId, ContextError> {
        match self.names.get(name) {
            Some(Sorts::Circle(circle)) => Ok(*circle),
            Some(Sorts::Point(_)) => Err(ContextError::ExpectedCircle(name.to_owned())),
            None => Err(ContextError::UnknownName(name.to_owned())),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::{HashMap, HashSet};

    use crate::{
        context::{Context, ContextError},
        engine::ConstructionError,
        types::{CircleId, Fact, Length, PointId, Sorts},
    };

    fn snapshot(
        context: &Context,
    ) -> (
        HashSet<Sorts>,
        HashSet<Fact>,
        HashSet<Fact>,
        HashMap<String, Sorts>,
    ) {
        (
            context.engine.known_objects.clone(),
            context.engine.facts.clone(),
            context.engine.assumptions.clone(),
            context.names.clone(),
        )
    }

    #[test]
    fn i1_construction_and_goal_check_use_fresh_named_objects() {
        let mut context = Context::default();
        let a = context.given_point("A").unwrap();
        let b = context.given_point("B").unwrap();
        assert_eq!((a, b), (PointId(0), PointId(1)));

        context.assume_distinct_points("A", "B").unwrap();
        assert!(
            context
                .engine
                .assumptions
                .contains(&Fact::DistinctPoints(a, b))
        );

        let alpha = context.construct_circle("alpha", "A", "B").unwrap();
        let beta = context.construct_circle("beta", "B", "A").unwrap();
        assert_eq!((alpha, beta), (CircleId(0), CircleId(1)));
        assert!(
            context
                .engine
                .contains(&Fact::CirclesIntersect(alpha, beta))
        );
        assert!(!context.names.contains_key("C"));
        assert!(
            !context
                .engine
                .known_objects
                .contains(&Sorts::Point(PointId(2)))
        );

        let c = context
            .construct_intersection("C", "alpha", "beta")
            .unwrap();
        assert_eq!(c, PointId(2));
        assert!(context.engine.contains(&Fact::OnCircle(c, alpha)));
        assert!(context.engine.contains(&Fact::OnCircle(c, beta)));
        assert!(
            context
                .engine
                .contains(&Fact::EqualLengths(Length::new(a, b), Length::new(b, c),))
        );
        assert!(
            context
                .engine
                .contains(&Fact::EqualLengths(Length::new(b, c), Length::new(c, a),))
        );
        assert!(!context.engine.contains(&Fact::DistinctPoints(a, c)));
        assert!(!context.engine.contains(&Fact::DistinctPoints(b, c)));

        let d = context.given_point("D").unwrap();
        assert_eq!(d, PointId(3));
    }

    #[test]
    fn failed_constructions_leave_state_and_allocators_unchanged() {
        let mut context = Context::default();
        let a = context.given_point("A").unwrap();
        let b = context.given_point("B").unwrap();
        let c = context.given_point("C").unwrap();
        let d = context.given_point("D").unwrap();
        assert_eq!(
            (a, b, c, d),
            (PointId(0), PointId(1), PointId(2), PointId(3))
        );

        let before = snapshot(&context);
        assert_eq!(
            context.construct_circle("alpha", "A", "B"),
            Err(ContextError::Construction(
                ConstructionError::PointsNotKnownDistinct(a, b)
            ))
        );
        assert_eq!(snapshot(&context), before);
        assert!(!context.names.contains_key("alpha"));

        context.assume_distinct_points("A", "B").unwrap();
        context.assume_distinct_points("C", "D").unwrap();
        let alpha = context.construct_circle("alpha", "A", "B").unwrap();
        let beta = context.construct_circle("beta", "C", "D").unwrap();
        assert_eq!((alpha, beta), (CircleId(0), CircleId(1)));
        assert!(
            !context
                .engine
                .contains(&Fact::CirclesIntersect(alpha, beta))
        );

        let before = snapshot(&context);
        assert_eq!(
            context.construct_intersection("X", "alpha", "beta"),
            Err(ContextError::Construction(
                ConstructionError::CirclesNotKnownIntersecting(alpha, beta)
            ))
        );
        assert_eq!(snapshot(&context), before);
        assert!(!context.names.contains_key("X"));

        let e = context.given_point("E").unwrap();
        assert_eq!(e, PointId(4));
    }

    #[test]
    fn name_errors_do_not_change_bindings_or_engine_state() {
        let mut context = Context::default();
        let a = context.given_point("A").unwrap();
        let b = context.given_point("B").unwrap();
        context.assume_distinct_points("A", "B").unwrap();
        context.construct_circle("alpha", "A", "B").unwrap();

        let before = snapshot(&context);
        assert_eq!(
            context.given_point("A"),
            Err(ContextError::DuplicateName("A".to_owned()))
        );
        assert_eq!(snapshot(&context), before);
        assert_eq!(
            context.construct_circle("gamma", "missing", "B"),
            Err(ContextError::UnknownName("missing".to_owned()))
        );
        assert_eq!(snapshot(&context), before);
        assert!(!context.names.contains_key("gamma"));
        assert_eq!(
            context.construct_circle("gamma", "alpha", "B"),
            Err(ContextError::ExpectedPoint("alpha".to_owned()))
        );
        assert_eq!(snapshot(&context), before);
        assert_eq!(
            context.construct_intersection("X", "A", "alpha"),
            Err(ContextError::ExpectedCircle("A".to_owned()))
        );
        assert_eq!(snapshot(&context), before);
        assert_eq!(
            context.construct_intersection("X", "missing", "alpha"),
            Err(ContextError::UnknownName("missing".to_owned()))
        );
        assert_eq!(snapshot(&context), before);
        assert_eq!(
            context.construct_intersection("alpha", "missing", "missing"),
            Err(ContextError::DuplicateName("alpha".to_owned()))
        );
        assert_eq!(snapshot(&context), before);

        let c = context.given_point("C").unwrap();
        assert_eq!(c, PointId(2));
        assert_ne!(a, b);
    }
}
