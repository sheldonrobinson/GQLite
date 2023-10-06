#!/usr/bin/env python3

import unittest as ut
import tempfile

import gqlite

class TestConnection(ut.TestCase):
  def test_simple_queries(self):
    fp = tempfile.NamedTemporaryFile()
    conn = gqlite.connect(fp.name)
    # Test one node creation
    self.assertEqual(conn.execute_oc_query("CREATE (n)"), None)
    # Test return
    self.assertEqual(conn.execute_oc_query("MATCH (n) RETURN n"), [['n'], [{'properties': {}, 'id': 1, 'labels': [], 'type': 'node'}]]) 
    # Test failure
    with self.assertRaises(gqlite.Error) as cm:
      conn.execute_oc_query("MATCH (n")
    self.assertEqual(cm.exception.msg, "CompileTime: UnexpectedSyntax: Expected token ) got end of file at (1, 8).")

if __name__ == '__main__':
    ut.main()
