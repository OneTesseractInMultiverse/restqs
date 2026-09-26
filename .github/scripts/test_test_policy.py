"""Regression tests for the Python assertion policy."""
import unittest

from test_policy import diagnostics


class AssertionPolicyTests(unittest.TestCase):
    def test_accepts_unittest_assertion(self):
        self.assertEqual(diagnostics("def test_ok(self): self.assertEqual(1, 1)"), [])

    def test_accepts_assert_statement(self):
        self.assertEqual(diagnostics("def test_ok(): assert True"), [])

    def test_detects_zero_assertions(self):
        self.assertEqual(diagnostics("def test_empty(): pass"),
                         [(1, 1, "test_empty: expected 1 assertion(s), found 0")])

    def test_detects_multiple_assertions(self):
        self.assertEqual(diagnostics("def test_many(self): self.assertTrue(True); assert True"),
                         [(1, 1, "test_many: expected 1 assertion(s), found 2")])

    def test_ignores_strings_and_comments(self):
        self.assertEqual(diagnostics('def test_text():\n s = "assert False" # assert False\n assert s'), [])

    def test_detects_helper_assertions(self):
        self.assertEqual(diagnostics("def setup(self): self.assertTrue(True)"),
                         [(1, 1, "setup: expected 0 assertion(s), found 1")])

    def test_counts_exception_context_manager(self):
        self.assertEqual(diagnostics("def test_error(self):\n with self.assertRaises(ValueError): fail()"), [])

    def test_nested_helper_does_not_satisfy_test(self):
        self.assertIn((1, 1, "test_outer: expected 1 assertion(s), found 0"),
                      diagnostics("def test_outer():\n def helper(): assert True"))

    def test_counts_mock_assertions(self):
        self.assertEqual(diagnostics("def test_mock(mock): mock.assert_called_once()"), [])

    def test_counts_async_tests(self):
        self.assertEqual(diagnostics("async def test_async(): assert True"), [])
