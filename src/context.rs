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
