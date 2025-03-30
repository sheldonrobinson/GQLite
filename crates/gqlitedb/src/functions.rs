use std::{collections::HashMap, fmt::Debug};

use crate::{aggregators, error, graph, Result};

mod containers;
mod edge;
mod node;
mod path;
mod scalar;
mod value;

pub(crate) type FResult<T> = std::result::Result<T, error::RunTimeError>;

use crate::interpreter::expression_analyser::ExpressionType;

pub(crate) trait FunctionTypeTrait
{
  fn result_type() -> ExpressionType;
}

impl FunctionTypeTrait for bool
{
  fn result_type() -> ExpressionType
  {
    ExpressionType::Boolean
  }
}

impl FunctionTypeTrait for String
{
  fn result_type() -> ExpressionType
  {
    ExpressionType::String
  }
}

impl FunctionTypeTrait for i64
{
  fn result_type() -> ExpressionType
  {
    ExpressionType::Integer
  }
}

impl<T> FunctionTypeTrait for Vec<T>
{
  fn result_type() -> ExpressionType
  {
    ExpressionType::Array
  }
}

//  _____                 _   _           _____          _ _
// |  ___|   _ _ __   ___| |_(_) ___  _ _|_   _| __ __ _(_) |_
// | |_ | | | | '_ \ / __| __| |/ _ \| '_ \| || '__/ _` | | __|
// |  _|| |_| | | | | (__| |_| | (_) | | | | || | | (_| | | |_
// |_|   \__,_|_| |_|\___|\__|_|\___/|_| |_|_||_|  \__,_|_|\__|

pub(crate) trait FunctionTrait: Debug
{
  fn call(&self, arguments: Vec<graph::Value>) -> Result<graph::Value>;
  fn validate_arguments(&self, arguments: Vec<ExpressionType>) -> Result<ExpressionType>;
  fn is_deterministic(&self) -> bool;
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

#[derive(Debug)]
struct ManagerInner
{
  functions: HashMap<String, Function>,
  aggregators: HashMap<String, aggregators::Aggregator>,
}

#[derive(Debug, Clone)]
pub(crate) struct Manager
{
  inner: std::rc::Rc<ManagerInner>,
}

impl Manager
{
  pub(crate) fn new() -> Self
  {
    Self {
      inner: std::rc::Rc::new(ManagerInner {
        functions: HashMap::from([
          containers::Keys::new(),
          containers::Range::new(),
          containers::Size::new(),
          edge::Type::new(),
          node::Labels::new(),
          path::Length::new(),
          scalar::Coalesce::new(),
          scalar::ToInteger::new(),
          value::HasLabel::new(),
          value::HasLabels::new(),
        ]),
        aggregators: aggregators::init_aggregators(),
      }),
    }
  }
  pub(crate) fn get_function<E: error::GenericErrors>(
    &self,
    name: impl Into<String>,
  ) -> Result<Function>
  {
    let name = name.into();
    Ok(
      self
        .inner
        .functions
        .get(&name)
        .ok_or_else(|| E::unknown_function(name).into())?
        .clone(),
    )
  }
  pub(crate) fn get_aggregator<E: error::GenericErrors>(
    &self,
    name: impl Into<String>,
  ) -> Result<aggregators::Aggregator>
  {
    let name = name.into();
    Ok(
      self
        .inner
        .aggregators
        .get(&name)
        .ok_or_else(|| E::unknown_function(name).into())?
        .clone(),
    )
  }
  pub(crate) fn is_deterministic(&self, name: impl Into<String>) -> Result<bool>
  {
    let name = name.into();
    let fun = self.get_function::<crate::error::CompileTimeError>(name.clone());
    match fun
    {
      Ok(fun) => Ok(fun.is_deterministic()),
      Err(_) =>
      {
        self.get_aggregator::<crate::error::CompileTimeError>(name)?;
        Ok(false)
      }
    }
  }
  pub(crate) fn is_aggregate(&self, name: &String) -> bool
  {
    self.inner.aggregators.contains_key(name)
  }

  pub(crate) fn validate_arguments(
    &self,
    name: impl Into<String>,
    arguments: Vec<ExpressionType>,
  ) -> Result<ExpressionType>
  {
    let name = name.into();
    let fun = self.get_function::<crate::error::CompileTimeError>(name.clone());
    match fun
    {
      Ok(fun) => fun.validate_arguments(arguments),
      Err(_) => self
        .get_aggregator::<crate::error::CompileTimeError>(name)?
        .validate_arguments(arguments),
    }
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
  ($function: expr, $arguments: ident, ) => {
    $function()
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
  ($function_name: ident, $type_name: ty, $f_name: ident (  $( $arg_type: ty $(,)? )* ) -> $ret_type: ty ) => {
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
            $crate::functions::make_function_call!(Self::$f_name, arguments, $( $arg_type,)*)
            .map(|r| -> graph::Value { r.into() })?,
          )
        }
        else
        {
          Err(RunTimeError::InvalidNumberOfArguments { function_name: stringify!($function_name), got: arguments.len(), expected: $crate::functions::count_arguments!($( $arg_type,)*) }.into())
        }
      }
      fn validate_arguments(
        &self,
        _: Vec<crate::interpreter::expression_analyser::ExpressionType>,
      ) -> crate::Result<crate::interpreter::expression_analyser::ExpressionType>
      {
        // TODO
        Ok(<$ret_type>::result_type())
      }
      fn is_deterministic(&self) -> bool
      {
        true
      }
    }
  };
  ($function_name: ident, $type_name: ty, custom_trait ) => {
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
  };
}

pub(crate) use count_arguments;
pub(crate) use declare_function;
pub(crate) use make_function_argument;
pub(crate) use make_function_call;
