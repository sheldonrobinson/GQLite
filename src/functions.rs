use std::{collections::HashMap, fmt::Debug};

use crate::{error, graph, Result};

mod edge;

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
      functions: HashMap::from([edge::Type::new()]),
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
