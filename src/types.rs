#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PointId(pub u32);

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CircleId(pub u32);

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Sorts {
    Point(PointId),
    Circle(CircleId),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Length {
    pub endpoints: [PointId; 2],
}

impl Length {
    pub fn new(a: PointId, b: PointId) -> Self {
        if a <= b {
            Self { endpoints: [a, b] }
        } else {
            Self { endpoints: [b, a] }
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Fact {
    DistinctPoints(PointId, PointId),
    CenterOfircle(PointId, CircleId),
    OnCircle(PointId, CircleId),
    InsideCircle(PointId, CircleId),
    CirclesIntersect(CircleId, CircleId),
    EqualLengths(Length, Length),
}

impl Fact {
    pub fn normalize(self) -> Self {
        match self {
            Self::DistinctPoints(a, b) if a > b => Self::DistinctPoints(b, a),
            Self::CirclesIntersect(a, b) if a > b => Self::CirclesIntersect(b, a),
            Self::EqualLengths(a, b) if a > b => Self::EqualLengths(b, a),
            fact => fact,
        }
    }
}
