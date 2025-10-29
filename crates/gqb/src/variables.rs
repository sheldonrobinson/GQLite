use crate::prelude::*;

/// Trait for passing variables to a function.
pub trait Variables
{
  /// Fill the vector of variables
  fn fill(self, vector: &mut Vec<Variable>);
}

impl Variables for Variable
{
  fn fill(self, vector: &mut Vec<Variable>)
  {
    vector.push(self);
  }
}

impl Variables for Vec<Variable>
{
  fn fill(mut self, vector: &mut Vec<Variable>)
  {
    vector.append(&mut self);
  }
}

macro_rules! impl_variables {
  ($n:tt $($idx:tt),*) => {
    impl Variables
      for ($(__key!($idx),)*)
    {
      fn fill(self, vector: &mut Vec<Variable>) {
        $(
          vector.push(self.$idx);
        )*
      }
    }
  };
}

// Generate implementations for 1..=20
macro_rules! impl_all_variables {
  ($($n:tt $($idx:tt),*;)*) => {
    $(impl_variables!($n $($idx),*);)*
  };
}

impl_all_variables! {
  2 0, 1;
  3 0, 1, 2;
  4 0, 1, 2, 3;
  5 0, 1, 2, 3, 4;
  6 0, 1, 2, 3, 4, 5;
  7 0, 1, 2, 3, 4, 5, 6;
  8 0, 1, 2, 3, 4, 5, 6, 7;
  9 0, 1, 2, 3, 4, 5, 6, 7, 8;
  10 0, 1, 2, 3, 4, 5, 6, 7, 8, 9;
  11 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10;
  12 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11;
  13 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12;
  14 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13;
  15 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14;
  16 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15;
  17 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16;
  18 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17;
  19 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18;
  20 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19;
}
