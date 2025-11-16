use crate::prelude::*;

/// Trait for multi-edge creation.
pub trait CreateEdges
{
  /// Output of multi-edge creation
  type Output;
  /// Fill the builder
  fn fill(self, builder: &mut Builder) -> Self::Output;
}

impl CreateEdges for Vec<(Variable, Vec<String>, graphcore::ValueMap, Variable)>
{
  type Output = Vec<Variable>;
  fn fill(self, builder: &mut Builder) -> Self::Output
  {
    let mut out = Vec::<Variable>::new();
    for (s, l, p, d) in self
    {
      out.push(builder.create_edge(s, l, p, d));
    }
    out
  }
}

macro_rules! impl_create_edges {
  ($n:tt $($idx:tt $s:ident $l:ident $p:ident $d:ident),*) => {
      impl<$($s, $l, $p, $d),*> CreateEdges
          for ($(($s, $l, $p, $d),)*)
      where
          $($s: Into<Variable>,
            $l: Into<Vec<String>>,
            $p: Into<graphcore::ValueMap>,
            $d: Into<Variable>),*
      {
          type Output = ($(__key!($idx),)*);

          fn fill(self, builder: &mut Builder) -> Self::Output {
              ($(
                  builder.create_edge(
                      (self.$idx).0,
                      (self.$idx).1,
                      (self.$idx).2,
                      (self.$idx).3,
                  ),
              )*)
          }
      }
  };
}

// Generate implementations for 1..=20
macro_rules! impl_all {
  ($($n:tt $($idx:tt $s:ident $l:ident $p:ident $d:ident),*;)*) => {
      $(impl_create_edges!($n $($idx $s $l $p $d),*);)*
  };
}

impl_all! {
  1 0 S0 L0 P0 D0;
  2 0 S0 L0 P0 D0, 1 S1 L1 P1 D1;
  3 0 S0 L0 P0 D0, 1 S1 L1 P1 D1, 2 S2 L2 P2 D2;
  4 0 S0 L0 P0 D0, 1 S1 L1 P1 D1, 2 S2 L2 P2 D2, 3 S3 L3 P3 D3;
  5 0 S0 L0 P0 D0, 1 S1 L1 P1 D1, 2 S2 L2 P2 D2, 3 S3 L3 P3 D3, 4 S4 L4 P4 D4;
  6 0 S0 L0 P0 D0, 1 S1 L1 P1 D1, 2 S2 L2 P2 D2, 3 S3 L3 P3 D3, 4 S4 L4 P4 D4, 5 S5 L5 P5 D5;
  7 0 S0 L0 P0 D0, 1 S1 L1 P1 D1, 2 S2 L2 P2 D2, 3 S3 L3 P3 D3, 4 S4 L4 P4 D4, 5 S5 L5 P5 D5, 6 S6 L6 P6 D6;
  8 0 S0 L0 P0 D0, 1 S1 L1 P1 D1, 2 S2 L2 P2 D2, 3 S3 L3 P3 D3, 4 S4 L4 P4 D4, 5 S5 L5 P5 D5, 6 S6 L6 P6 D6, 7 S7 L7 P7 D7;
  9 0 S0 L0 P0 D0, 1 S1 L1 P1 D1, 2 S2 L2 P2 D2, 3 S3 L3 P3 D3, 4 S4 L4 P4 D4, 5 S5 L5 P5 D5, 6 S6 L6 P6 D6, 7 S7 L7 P7 D7, 8 S8 L8 P8 D8;
  10 0 S0 L0 P0 D0, 1 S1 L1 P1 D1, 2 S2 L2 P2 D2, 3 S3 L3 P3 D3, 4 S4 L4 P4 D4, 5 S5 L5 P5 D5, 6 S6 L6 P6 D6, 7 S7 L7 P7 D7, 8 S8 L8 P8 D8, 9 S9 L9 P9 D9;
  11 0 S0 L0 P0 D0, 1 S1 L1 P1 D1, 2 S2 L2 P2 D2, 3 S3 L3 P3 D3, 4 S4 L4 P4 D4, 5 S5 L5 P5 D5, 6 S6 L6 P6 D6, 7 S7 L7 P7 D7, 8 S8 L8 P8 D8, 9 S9 L9 P9 D9, 10 S10 L10 P10 D10;
  12 0 S0 L0 P0 D0, 1 S1 L1 P1 D1, 2 S2 L2 P2 D2, 3 S3 L3 P3 D3, 4 S4 L4 P4 D4, 5 S5 L5 P5 D5, 6 S6 L6 P6 D6, 7 S7 L7 P7 D7, 8 S8 L8 P8 D8, 9 S9 L9 P9 D9, 10 S10 L10 P10 D10, 11 S11 L11 P11 D11;
  13 0 S0 L0 P0 D0, 1 S1 L1 P1 D1, 2 S2 L2 P2 D2, 3 S3 L3 P3 D3, 4 S4 L4 P4 D4, 5 S5 L5 P5 D5, 6 S6 L6 P6 D6, 7 S7 L7 P7 D7, 8 S8 L8 P8 D8, 9 S9 L9 P9 D9, 10 S10 L10 P10 D10, 11 S11 L11 P11 D11, 12 S12 L12 P12 D12;
  14 0 S0 L0 P0 D0, 1 S1 L1 P1 D1, 2 S2 L2 P2 D2, 3 S3 L3 P3 D3, 4 S4 L4 P4 D4, 5 S5 L5 P5 D5, 6 S6 L6 P6 D6, 7 S7 L7 P7 D7, 8 S8 L8 P8 D8, 9 S9 L9 P9 D9, 10 S10 L10 P10 D10, 11 S11 L11 P11 D11, 12 S12 L12 P12 D12, 13 S13 L13 P13 D13;
  15 0 S0 L0 P0 D0, 1 S1 L1 P1 D1, 2 S2 L2 P2 D2, 3 S3 L3 P3 D3, 4 S4 L4 P4 D4, 5 S5 L5 P5 D5, 6 S6 L6 P6 D6, 7 S7 L7 P7 D7, 8 S8 L8 P8 D8, 9 S9 L9 P9 D9, 10 S10 L10 P10 D10, 11 S11 L11 P11 D11, 12 S12 L12 P12 D12, 13 S13 L13 P13 D13, 14 S14 L14 P14 D14;
  16 0 S0 L0 P0 D0, 1 S1 L1 P1 D1, 2 S2 L2 P2 D2, 3 S3 L3 P3 D3, 4 S4 L4 P4 D4, 5 S5 L5 P5 D5, 6 S6 L6 P6 D6, 7 S7 L7 P7 D7, 8 S8 L8 P8 D8, 9 S9 L9 P9 D9, 10 S10 L10 P10 D10, 11 S11 L11 P11 D11, 12 S12 L12 P12 D12, 13 S13 L13 P13 D13, 14 S14 L14 P14 D14, 15 S15 L15 P15 D15;
  17 0 S0 L0 P0 D0, 1 S1 L1 P1 D1, 2 S2 L2 P2 D2, 3 S3 L3 P3 D3, 4 S4 L4 P4 D4, 5 S5 L5 P5 D5, 6 S6 L6 P6 D6, 7 S7 L7 P7 D7, 8 S8 L8 P8 D8, 9 S9 L9 P9 D9, 10 S10 L10 P10 D10, 11 S11 L11 P11 D11, 12 S12 L12 P12 D12, 13 S13 L13 P13 D13, 14 S14 L14 P14 D14, 15 S15 L15 P15 D15, 16 S16 L16 P16 D16;
  18 0 S0 L0 P0 D0, 1 S1 L1 P1 D1, 2 S2 L2 P2 D2, 3 S3 L3 P3 D3, 4 S4 L4 P4 D4, 5 S5 L5 P5 D5, 6 S6 L6 P6 D6, 7 S7 L7 P7 D7, 8 S8 L8 P8 D8, 9 S9 L9 P9 D9, 10 S10 L10 P10 D10, 11 S11 L11 P11 D11, 12 S12 L12 P12 D12, 13 S13 L13 P13 D13, 14 S14 L14 P14 D14, 15 S15 L15 P15 D15, 16 S16 L16 P16 D16, 17 S17 L17 P17 D17;
  19 0 S0 L0 P0 D0, 1 S1 L1 P1 D1, 2 S2 L2 P2 D2, 3 S3 L3 P3 D3, 4 S4 L4 P4 D4, 5 S5 L5 P5 D5, 6 S6 L6 P6 D6, 7 S7 L7 P7 D7, 8 S8 L8 P8 D8, 9 S9 L9 P9 D9, 10 S10 L10 P10 D10, 11 S11 L11 P11 D11, 12 S12 L12 P12 D12, 13 S13 L13 P13 D13, 14 S14 L14 P14 D14, 15 S15 L15 P15 D15, 16 S16 L16 P16 D16, 17 S17 L17 P17 D17, 18 S18 L18 P18 D18;
  20 0 S0 L0 P0 D0, 1 S1 L1 P1 D1, 2 S2 L2 P2 D2, 3 S3 L3 P3 D3, 4 S4 L4 P4 D4, 5 S5 L5 P5 D5, 6 S6 L6 P6 D6, 7 S7 L7 P7 D7, 8 S8 L8 P8 D8, 9 S9 L9 P9 D9, 10 S10 L10 P10 D10, 11 S11 L11 P11 D11, 12 S12 L12 P12 D12, 13 S13 L13 P13 D13, 14 S14 L14 P14 D14, 15 S15 L15 P15 D15, 16 S16 L16 P16 D16, 17 S17 L17 P17 D17, 18 S18 L18 P18 D18, 19 S19 L19 P19 D19;
}
