use std::{
    collections::{HashMap, HashSet},
    fmt,
};

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

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum NamedFact {
    DistinctPoints(String, String),
    EqualLengths(String, String, String, String),
}

impl fmt::Display for NamedFact {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::DistinctPoints(a, b) => write!(formatter, "distinct {a} {b}"),
            Self::EqualLengths(a, b, c, d) => write!(formatter, "equal {a} {b} {c} {d}"),
        }
    }
}

#[derive(Default)]
pub struct Context {
    pub engine: Engine,
    pub names: HashMap<String, Sorts>,
    pub conclusions: HashSet<Fact>,
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

    pub fn known_fact(&self, named_fact: &NamedFact) -> Result<Option<Fact>, ContextError> {
        let fact = match named_fact {
            NamedFact::DistinctPoints(a, b) => {
                Fact::DistinctPoints(self.resolve_point(a)?, self.resolve_point(b)?)
            }
            NamedFact::EqualLengths(a, b, c, d) => Fact::EqualLengths(
                Length::new(self.resolve_point(a)?, self.resolve_point(b)?),
                Length::new(self.resolve_point(c)?, self.resolve_point(d)?),
            ),
        }
        .normalize();

        Ok(self.engine.contains(&fact).then_some(fact))
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
