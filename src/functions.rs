use std::{collections::HashMap, fmt::Debug};

use crate::{error, graph, Result};

mod edge;
mod value;

pub(crate) type FResult<T> = std::result::Result<T, error::RunTimeError>;

//  _____                 _   _           _____          _ _
// |  ___|   _ _ __   ___| |_(_) ___  _ _|_   _| __ __ _(_) |_
// | |_ | | | | '_ \ / __| __| |/ _ \| '_ \| || '__/ _` | | __|
// |  _|| |_| | | | | (__| |_| | (_) | | | | || | | (_| | | |_
// |_|   \__,_|_| |_|\___|\__|_|\___/|_| |_|_||_|  \__,_|_|\__|

pub(crate) trait FunctionTrait: Debug
{
  fn call(&self, arguments: Vec<graph::Value>) -> Result<graph::Value>;
}

//  _____                 _   _
// |  ___|   _ _ __   ___| |_(_) ___  _ __
// | |_ | | | | '_ \ / __| __| |/ _ \| '_ \
// |  _|| |_| | | | | (__| |_| | (_) | | | |
// |_|   \__,_|_| |_|\___|\__|_|\___/|_| |_|

pub(crate) type Function = std::rc::Rc<Box<dyn FunctionTrait>>;

//  __  __
// |  \/  | __ _ _ __   __ _  __ _  ___ _ __
// | |\/| |/ _` | '_ \ / _` |/ _` |/ _ \ '__|
// | |  | | (_| | | | | (_| | (_| |  __/ |
// |_|  |_|\__,_|_| |_|\__,_|\__, |\___|_|
//                           |___/

pub(crate) struct Manager
{
  functions: HashMap<String, Function>,
}

impl Manager
{
  pub(crate) fn new() -> Self
  {
    Self {
      functions: HashMap::from([edge::Type::new(), value::HasLabel::new()]),
    }
  }
  pub(crate) fn get<E: error::GenericErrors>(&self, name: impl Into<String>) -> Result<Function>
  {
    let name = name.into();
    Ok(
      self
        .functions
        .get(&name)
        .ok_or_else(|| E::unknown_function(name).into())?
        .clone(),
    )
  }
}

macro_rules! make_function_argument {
  ($arguments: ident, $index: expr, $arg_type: ty) => {
    $arguments[$index]
      .try_into_ref()
      .map_err(|_| RunTimeError::InvalidArgument {
        function_name: stringify!($function_name),
        index: $index,
        expected_type: stringify!($arg_type),
        value: format!("{:?}", $arguments[$index]),
      })?
  };
}

macro_rules! make_function_call {
  ($function: expr, $arguments: ident, $arg_type_0: ty, ) => {
    $function($crate::functions::make_function_argument!(
      $arguments,
      0,
      $arg_type_0
    ))
  };
  ($function: expr, $arguments: ident, $arg_type_0: ty, $arg_type_1: ty, ) => {
    $function(
      $crate::functions::make_function_argument!($arguments, 0, $arg_type_0),
      $crate::functions::make_function_argument!($arguments, 1, $arg_type_1),
    )
  };
}

macro_rules! count_arguments {
  ($count: expr, ) => {
    $count
  };
  ($count: expr, $arg_type_0: ty, $( $arg_type: ty , )*) => {
    $crate::functions::count_arguments!($count + 1, $( $arg_type, )*)
  };
  ($( $arg_type: ty $(,)? )* ) => {
    $crate::functions::count_arguments!(0, $( $arg_type, )*)
  };
}

macro_rules! declare_function {
  ($function_name: ident, $type_name: ty, ( $( $arg_type: ty $(,)? )* ) ) => {
    impl $type_name
    {
      pub(super) fn new() -> (String, crate::functions::Function)
      {
        (
          stringify!($function_name).to_string(),
          std::rc::Rc::new(Box::new(Self {})),
        )
      }
    }
    impl crate::functions::FunctionTrait for $type_name
    {
      fn call(&self, arguments: Vec<crate::graph::Value>) -> crate::Result<crate::graph::Value>
      {
        if arguments.len() == $crate::functions::count_arguments!($( $arg_type,)*)
        {
          use crate::graph::ValueTryIntoRef;
          Ok(
            $crate::functions::make_function_call!(Self::call_impl, arguments, $( $arg_type,)*)
            .map(|r| -> graph::Value { r.into() })?,
          )
        }
        else
        {
          Err(RunTimeError::InvalidNumberOfArguments { function_name: stringify!($function_name), got: arguments.len(), expected: $crate::functions::count_arguments!($( $arg_type,)*) }.into())
        }
      }
    }
  };
}

pub(crate) use count_arguments;
pub(crate) use declare_function;
pub(crate) use make_function_argument;
pub(crate) use make_function_call;
