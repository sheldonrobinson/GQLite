SELECT count(*) FROM sqlite_master
  WHERE type='table'
    AND (name='gqlite_/%= _name %/_nodes' or name='gqlite_/%= _name %/_edges' or name='gqlite_/%= _name %/_labels')