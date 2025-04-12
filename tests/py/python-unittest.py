#!/usr/bin/env python3

import unittest as ut
import re
import tempfile

import gqlite


class TestConnection(ut.TestCase):
    def test_simple_queries(self):
        fp = tempfile.NamedTemporaryFile()
        conn = gqlite.connect(fp.name)
        # Test one node creation
        self.assertEqual(conn.execute_oc_query("CREATE (n)"), None)
        # Test return
        qr = conn.execute_oc_query("MATCH (n) RETURN n")
        del qr[1][0]["key"]
        self.assertEqual(
            qr, [["n"], [{"properties": {}, "labels": [], "type": "node"}]]
        )
        # Test failure
        with self.assertRaises(gqlite.Error) as cm:
            conn.execute_oc_query("MATCH (n")
        self.assertTrue(
            re.match("CompileTime: ParseError: .*", cm.exception.msg),
        )


if __name__ == "__main__":
    ut.main()
