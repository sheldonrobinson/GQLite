use crate::prelude::*;

/// Trait for multi-node creation.
pub trait CreateNodes
{
  /// Output of multi-node creation
  type Output;
  /// Fill the builder
  fn fill(self, builder: &mut Builder) -> Self::Output;
}

macro_rules! impl_create_nodes {
  ($n:tt $($idx:tt $l:ident $p:ident),*) => {
      impl<$($l, $p),*> CreateNodes
          for ($(($l, $p),)*)
      where
          $($l: Into<Vec<String>>,
            $p: Into<graphcore::ValueMap>),*
      {
          type Output = ($(__key!($idx),)*);

          fn fill(self, builder: &mut Builder) -> Self::Output {
              ($(
                  builder.create_node(
                      (self.$idx).0,
                      (self.$idx).1,
                  ),
              )*)
          }
      }
  };
}

// Generate implementations for 1..=20
macro_rules! impl_all_create_nodes {
  ($($n:tt $($idx:tt $l:ident $p:ident),*;)*) => {
      $(impl_create_nodes!($n $($idx $l $p),*);)*
  };
}

impl_all_create_nodes! {
  1 0 L0 P0;
  2 0 L0 P0, 1 L1 P1;
  3 0 L0 P0, 1 L1 P1, 2 L2 P2;
  4 0 L0 P0, 1 L1 P1, 2 L2 P2, 3 L3 P3;
  5 0 L0 P0, 1 L1 P1, 2 L2 P2, 3 L3 P3, 4 L4 P4;
  6 0 L0 P0, 1 L1 P1, 2 L2 P2, 3 L3 P3, 4 L4 P4, 5 L5 P5;
  7 0 L0 P0, 1 L1 P1, 2 L2 P2, 3 L3 P3, 4 L4 P4, 5 L5 P5, 6 L6 P6;
  8 0 L0 P0, 1 L1 P1, 2 L2 P2, 3 L3 P3, 4 L4 P4, 5 L5 P5, 6 L6 P6, 7 L7 P7;
  9 0 L0 P0, 1 L1 P1, 2 L2 P2, 3 L3 P3, 4 L4 P4, 5 L5 P5, 6 L6 P6, 7 L7 P7, 8 L8 P8;
  10 0 L0 P0, 1 L1 P1, 2 L2 P2, 3 L3 P3, 4 L4 P4, 5 L5 P5, 6 L6 P6, 7 L7 P7, 8 L8 P8, 9 L9 P9;
  11 0 L0 P0, 1 L1 P1, 2 L2 P2, 3 L3 P3, 4 L4 P4, 5 L5 P5, 6 L6 P6, 7 L7 P7, 8 L8 P8, 9 L9 P9, 10 L10 P10;
  12 0 L0 P0, 1 L1 P1, 2 L2 P2, 3 L3 P3, 4 L4 P4, 5 L5 P5, 6 L6 P6, 7 L7 P7, 8 L8 P8, 9 L9 P9, 10 L10 P10, 11 L11 P11;
  13 0 L0 P0, 1 L1 P1, 2 L2 P2, 3 L3 P3, 4 L4 P4, 5 L5 P5, 6 L6 P6, 7 L7 P7, 8 L8 P8, 9 L9 P9, 10 L10 P10, 11 L11 P11, 12 L12 P12;
  14 0 L0 P0, 1 L1 P1, 2 L2 P2, 3 L3 P3, 4 L4 P4, 5 L5 P5, 6 L6 P6, 7 L7 P7, 8 L8 P8, 9 L9 P9, 10 L10 P10, 11 L11 P11, 12 L12 P12, 13 L13 P13;
  15 0 L0 P0, 1 L1 P1, 2 L2 P2, 3 L3 P3, 4 L4 P4, 5 L5 P5, 6 L6 P6, 7 L7 P7, 8 L8 P8, 9 L9 P9, 10 L10 P10, 11 L11 P11, 12 L12 P12, 13 L13 P13, 14 L14 P14;
  16 0 L0 P0, 1 L1 P1, 2 L2 P2, 3 L3 P3, 4 L4 P4, 5 L5 P5, 6 L6 P6, 7 L7 P7, 8 L8 P8, 9 L9 P9, 10 L10 P10, 11 L11 P11, 12 L12 P12, 13 L13 P13, 14 L14 P14, 15 L15 P15;
  17 0 L0 P0, 1 L1 P1, 2 L2 P2, 3 L3 P3, 4 L4 P4, 5 L5 P5, 6 L6 P6, 7 L7 P7, 8 L8 P8, 9 L9 P9, 10 L10 P10, 11 L11 P11, 12 L12 P12, 13 L13 P13, 14 L14 P14, 15 L15 P15, 16 L16 P16;
  18 0 L0 P0, 1 L1 P1, 2 L2 P2, 3 L3 P3, 4 L4 P4, 5 L5 P5, 6 L6 P6, 7 L7 P7, 8 L8 P8, 9 L9 P9, 10 L10 P10, 11 L11 P11, 12 L12 P12, 13 L13 P13, 14 L14 P14, 15 L15 P15, 16 L16 P16, 17 L17 P17;
  19 0 L0 P0, 1 L1 P1, 2 L2 P2, 3 L3 P3, 4 L4 P4, 5 L5 P5, 6 L6 P6, 7 L7 P7, 8 L8 P8, 9 L9 P9, 10 L10 P10, 11 L11 P11, 12 L12 P12, 13 L13 P13, 14 L14 P14, 15 L15 P15, 16 L16 P16, 17 L17 P17, 18 L18 P18;
  20 0 L0 P0, 1 L1 P1, 2 L2 P2, 3 L3 P3, 4 L4 P4, 5 L5 P5, 6 L6 P6, 7 L7 P7, 8 L8 P8, 9 L9 P9, 10 L10 P10, 11 L11 P11, 12 L12 P12, 13 L13 P13, 14 L14 P14, 15 L15 P15, 16 L16 P16, 17 L17 P17, 18 L18 P18, 19 L19 P19;
}

impl CreateNodes for Vec<(Vec<String>, graphcore::ValueMap)>
{
  type Output = Vec<Variable>;
  fn fill(self, builder: &mut Builder) -> Self::Output
  {
    let mut out = Vec::<Variable>::new();
    for (l, p) in self
    {
      out.push(builder.create_node(l, p))
    }
    out
  }
}
