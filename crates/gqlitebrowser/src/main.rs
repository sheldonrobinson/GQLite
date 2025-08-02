#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use gqlitedb::ValueTryIntoRef as _;

/// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)] // if we add new fields, give them default values when deserializing old state
pub struct GqliteBrowser
{
  history: Vec<String>,
  query_text: String,
  last_message: String,
  result_delegate: ResultDelegate,
  #[serde(skip)]
  connection_error: String,
  #[serde(skip)]
  connection: Option<gqlitedb::Connection>,
}

impl Default for GqliteBrowser
{
  fn default() -> Self
  {
    Self {
      history: Default::default(),
      query_text: Default::default(),
      last_message: Default::default(),
      result_delegate: ResultDelegate {
        results: Default::default(),
      },
      connection_error: String::new(),
      connection: None,
    }
  }
}

impl GqliteBrowser
{
  /// Called once before the first frame.
  pub fn new(cc: &eframe::CreationContext<'_>) -> Self
  {
    egui_material_icons::initialize(&cc.egui_ctx);
    // Load previous app state (if any).
    // Note that you must enable the `persistence` feature for this to work.
    if let Some(storage) = cc.storage
    {
      eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default()
    }
    else
    {
      Default::default()
    }
  }
  fn execute_query(&mut self)
  {
    match &self.connection
    {
      Some(connection) =>
      {
        self.history.retain(|v| *v != self.query_text);
        self.history.push(self.query_text.to_owned());

        let result = connection.execute_query(self.query_text.to_owned(), Default::default());
        self.last_message.clear();
        match result
        {
          Ok(t) => match t
          {
            gqlitedb::Value::Array(arr) =>
            {
              self.result_delegate.results = arr;
            }
            _ =>
            {
              self.result_delegate.results.clear();
            }
          },
          Err(err) =>
          {
            self.last_message = match err.error()
            {
              gqlitedb::Error::CompileTime(ct) =>
              {
                format!("Compilation error:\n{}", ct.to_string())
              }
              _ =>
              {
                format!("Query execution failed: {:?}", err)
              }
            }
          }
        }
      }
      None =>
      {}
    }
  }
  fn set_connection(&mut self, connection: gqlitedb::Result<gqlitedb::Connection>)
  {
    match connection
    {
      Ok(connection) => self.connection = Some(connection),
      Err(e) =>
      {
        self.connection = None;
        self.connection_error = format!("{:?}", e);
      }
    }
  }
  fn create_connection(&mut self, backend: impl Into<String>)
  {
    self.set_connection(gqlitedb::Connection::create(
      gqlitedb::map! {"backend" => backend.into()},
    ));
  }
  #[cfg(not(target_arch = "wasm32"))]
  fn new_database(&mut self, backend: impl Into<String>)
  {
    let filename = rfd::FileDialog::new().save_file().unwrap();
    if std::fs::exists(&filename).unwrap()
    {
      self.connection = None;
      self.connection_error = format!("'{:?}' already exists, did you want to open?", filename);
    }
    else
    {
      let connection = gqlitedb::Connection::builder()
        .options(gqlitedb::map! {"backend" => backend.into()})
        .path(filename)
        .create();
      self.set_connection(connection);
    }
  }

  #[cfg(not(target_arch = "wasm32"))]
  fn open_database(&mut self)
  {
    let filename = rfd::FileDialog::new().pick_file().unwrap();
    if std::fs::exists(&filename).unwrap()
    {
      let connection = gqlitedb::Connection::builder().path(filename).create();
      self.set_connection(connection);
    }
    else
    {
      self.connection = None;
      self.connection_error = format!("'{:?}' does not exists.", filename);
    }
  }
  #[cfg(not(target_arch = "wasm32"))]
  fn support_opening() -> bool
  {
    true
  }
  #[cfg(target_arch = "wasm32")]
  fn new_database(&mut self, _backend: impl Into<String>) {}
  #[cfg(target_arch = "wasm32")]
  fn open_database(&mut self) {}
  #[cfg(target_arch = "wasm32")]
  fn support_opening() -> bool
  {
    false
  }
}

brisk_eframe::brisk_it! {
  Main
  {
      title: "GQLite Browser",
      viewport: Viewport
      {
          inner_size: [320.0, 240.0],
      },
      App {
          target: GqliteBrowser,
          save: true,
          TopBottomPanel {
              id: top,
              position: Top,
              MenuBar {
                  MenuButton {
                    text: "Connection",
                    Button {
                      text: "New sqlite (in-memory)",
                      visible: gqlitedb::Connection::available_backends().contains(&"sqlite".to_string()),
                      on_clicked: self.create_connection("sqlite")
                    },
                    Button {
                      text: "New redb (in-memory)",
                      visible: gqlitedb::Connection::available_backends().contains(&"redb".to_string()),
                      on_clicked: self.create_connection("redb")
                    },
                    Button {
                      text: "New sqlite (file)",
                      visible: gqlitedb::Connection::available_backends().contains(&"sqlite".to_string()) && Self::support_opening(),
                      on_clicked: self.new_database("sqlite")
                    },
                    Button {
                      text: "New redb (file)",
                      visible: gqlitedb::Connection::available_backends().contains(&"redb".to_string()) && Self::support_opening(),
                      on_clicked: self.new_database("redb")
                    },
                    Button {
                      text: "Open (file)",
                      visible: gqlitedb::Connection::available_backends().contains(&"sqlite".to_string()) && Self::support_opening(),
                      on_clicked: self.open_database()
                    },
                  }
              },
          },
          CentralPanel
          {
            visible: self.connection.is_none(),
            Label
            {
              visible: self.connection_error.is_empty(),
              text: "No connection...",
            },
            Label
            {
              visible: !self.connection_error.is_empty(),
              text: format!("{} {}", egui_material_icons::icons::ICON_ERROR, self.connection_error)
            }
          },
          CentralPanel
          {
              visible: self.connection.is_some(),
              Horizontal
              {
                  Label
                  {
                      id: name_label,
                      text: "Prompt: "
                  },
                  TextEdit
                  {
                      text: &mut self.query_text,
                      labelled_by: name_label.id,
                      hint_text: "Write an OpenCypher query, and then press enter to execute.",
                      on_editing_finished: { self.execute_query(); }
                  },
                  Button
                  {
                      text: "history ⬇",
                      PopupMenu {
                          Repeater
                          {
                              target: query,
                              model: self.history.iter().rev(),
                              Button {
                                text: query,
                                on_clicked: self.query_text = query.to_owned()
                              }
                          }
                      }
                  }
              },
              Label
              {
                visible: !self.last_message.is_empty(),
                text: format!("{} {}", egui_material_icons::icons::ICON_ERROR, self.last_message)
              },
              Table {
                  visible: !self.result_delegate.results.is_empty(),
                  table_delegate:  &mut self.result_delegate,
                  num_rows: self.result_delegate.num_rows(),
                  columns: vec![egui_table::Column::new(100.0)
                          .range(10.0..=500.0)
                          .resizable(true); self.result_delegate.num_columns()],
                  auto_size_mode: egui_table::AutoSizeMode::OnParentResize,
                  headers: [
                      HeaderRow {
                          height: 24.0,
                      }
                  ]
              }
          }
      }
  }
}

#[derive(serde::Deserialize, serde::Serialize)]
struct ResultDelegate
{
  results: Vec<gqlitedb::Value>,
}

impl ResultDelegate
{
  fn num_rows(&self) -> u64
  {
    self.results.len().saturating_sub(1) as u64
  }
  fn num_columns(&self) -> usize
  {
    match self.results.len()
    {
      0 => 0,
      _ => match &self.results[0]
      {
        gqlitedb::Value::Array(arr) => arr.len(),
        _ => 1,
      },
    }
  }
  fn cell_content_ui(&mut self, row_nr: u64, col_nr: usize, ui: &mut egui::Ui)
  {
    let row: &Vec<gqlitedb::Value> = self.results[row_nr as usize + 1].try_into_ref().unwrap();
    brisk_egui::brisk_it! {
      Label
      {
        text: format!("{}", row[col_nr as usize]),
      }
    }
  }
}

impl egui_table::TableDelegate for ResultDelegate
{
  fn header_cell_ui(&mut self, ui: &mut egui::Ui, cell_inf: &egui_table::HeaderCellInfo)
  {
    let egui_table::HeaderCellInfo { group_index, .. } = cell_inf;

    let margin = 4;

    let row: &Vec<gqlitedb::Value> = self.results[0].try_into_ref().unwrap();

    egui::Frame::NONE
      .inner_margin(egui::Margin::symmetric(margin, 0))
      .show(ui, |ui| {
        ui.heading(format!("{}", row[*group_index as usize]));
      });
  }
  fn cell_ui(&mut self, ui: &mut egui::Ui, cell_info: &egui_table::CellInfo)
  {
    let egui_table::CellInfo { row_nr, col_nr, .. } = *cell_info;

    if row_nr % 2 == 1
    {
      ui.painter()
        .rect_filled(ui.max_rect(), 0.0, ui.visuals().faint_bg_color);
    }

    egui::Frame::NONE
      .inner_margin(egui::Margin::symmetric(4, 0))
      .show(ui, |ui| {
        self.cell_content_ui(row_nr, col_nr, ui);
      });
  }
}
