#!/usr/bin/env python3
"""Manual read-only TITrack probe; stdlib only, Python 3.10+ (including Windows).

Never executes downloaded Python, never prints a client key, never uploads game
information. CI may run --self-test; CI does not call the real Supabase endpoint.
"""
import argparse
import ast
import datetime
import email.message
import json
from pathlib import Path
import re
import unittest
import urllib.error
import urllib.parse
import urllib.request

HOST = "https://qhjulyngunwiculnharg.supabase.co"
HEAD_URL = "https://api.github.com/repos/astockman99/TITrack/commits/master"
FIELDS = ("config_base_id", "season_id", "price_fe_median", "price_fe_p10",
          "price_fe_p90", "submission_count", "unique_devices", "updated_at")
TIMEOUT = 15


class NoRedirect(urllib.request.HTTPRedirectHandler):
    def redirect_request(self, req, fp, code, msg, headers, newurl):
        raise RuntimeError("HTTP redirect refused; expected hosts are fixed")


def read_response(response, maximum):
    body = response.read(maximum + 1)
    if len(body) > maximum:
        raise ValueError("Response exceeds diagnostic size limit")
    return body


def public_configuration(source):
    """Read literal declarations, without importing/executing TITrack code."""
    values = {}
    for node in ast.walk(ast.parse(source)):
        if isinstance(node, ast.Assign) and isinstance(node.value, ast.Constant):
            for target in node.targets:
                if isinstance(target, ast.Name) and target.id in (
                        "DEFAULT_SUPABASE_URL", "DEFAULT_SUPABASE_KEY"):
                    if target.id in values:
                        raise ValueError("Ambiguous public configuration; stopped")
                    values[target.id] = node.value.value
    key = values.get("DEFAULT_SUPABASE_KEY", "")
    if values.get("DEFAULT_SUPABASE_URL") != HOST or not isinstance(key, str) or not re.fullmatch(
            r"sb_publishable_[A-Za-z0-9_-]+", key):
        raise ValueError("Expected host and public publishable key not found; stopped")
    return key


def safe_headers(headers):
    names = {"content-type", "content-range", "retry-after", "server", "date",
             "ratelimit-limit", "ratelimit-remaining", "ratelimit-reset",
             "x-ratelimit-limit", "x-ratelimit-remaining", "x-ratelimit-reset"}
    return {k.lower(): v for k, v in headers.items() if k.lower() in names}


def probe(open_url=None):
    if open_url is None:
        open_url = urllib.request.build_opener(NoRedirect()).open
    result = {"checked_at_utc": datetime.datetime.now(datetime.timezone.utc).isoformat(),
              "host": HOST, "method": "GET", "season_id": 1601, "limit": 1,
              "selected_fields": list(FIELDS), "phase": "github_head",
              "server_http_status": None, "key_printed": False}
    key = ""
    try:
        request = urllib.request.Request(HEAD_URL, headers={"Accept": "application/vnd.github+json", "User-Agent": "Deanfinite-readonly-diagnostic/1"})
        with open_url(request, timeout=TIMEOUT) as response:
            metadata = json.loads(read_response(response, 256 * 1024))
        if not isinstance(metadata, dict):
            raise ValueError("Expected GitHub commit object")
        revision = metadata.get("sha", "")
        if not re.fullmatch(r"[0-9a-f]{40}", revision):
            raise ValueError("Invalid TITrack revision")
        result["source_commit"] = revision
        result["phase"] = "github_public_configuration"
        source_url = f"https://raw.githubusercontent.com/astockman99/TITrack/{revision}/src/titrack/sync/client.py"
        with open_url(urllib.request.Request(source_url, headers={"User-Agent": "Deanfinite-readonly-diagnostic/1"}), timeout=TIMEOUT) as response:
            key = public_configuration(read_response(response, 256 * 1024).decode("utf-8"))
        result["phase"] = "supabase"
        url = HOST + "/rest/v1/aggregated_prices?" + urllib.parse.urlencode({
            "select": ",".join(FIELDS), "season_id": "eq.1601", "limit": "1", "order": "config_base_id.asc"})
        # Count is optional server metadata on the SAME one-row read, not another
        # endpoint or a bulk table export. A missing total remains unconfirmed.
        request = urllib.request.Request(url, headers={"apikey": key, "Accept": "application/json", "Prefer": "count=exact", "User-Agent": "Deanfinite-readonly-diagnostic/1"}, method="GET")
        with open_url(request, timeout=TIMEOUT) as response:
            result["server_http_status"] = response.status
            result["headers"] = safe_headers(response.headers)
            rows = json.loads(read_response(response, 64 * 1024))
        if not isinstance(rows, list) or len(rows) > 1 or any(not isinstance(row, dict) for row in rows):
            raise ValueError("Expected JSON array with at most one row")
        result["anonymous_select_confirmed"] = result["server_http_status"] in (200, 206)
        result["ss14_rows_found"] = any(row.get("season_id") == 1601 for row in rows)
        result["rows_returned"] = len(rows)
        result["returned_fields"] = sorted(rows[0]) if rows else []
        # Only selected market columns are included; no arbitrary response fields.
        result["sample"] = [{field: row.get(field) for field in FIELDS if field in row} for row in rows]
        if any(row.get("season_id") != 1601 for row in rows):
            raise ValueError("Response contains unexpected season; SS14 coverage not confirmed")
        match = re.fullmatch(r"(?:[0-9]+-[0-9]+|\*)/([0-9]+)", result["headers"].get("content-range", ""))
        result["total_ids_if_reported"] = int(match[1]) if match else None
    except urllib.error.HTTPError as error:
        if result["phase"] == "supabase":
            result["server_http_status"] = error.code
            result["headers"] = safe_headers(error.headers)
        result["error"] = f"HTTP {error.code} in {result['phase']}; no retry was attempted"
    except (urllib.error.URLError, TimeoutError, OSError, ValueError, RuntimeError, SyntaxError) as error:
        result["transport_or_format_error"] = str(error)
    # Redact even a key accidentally echoed by a remote error/selected column.
    serialized = json.dumps(result, ensure_ascii=False)
    if key:
        serialized = serialized.replace(key, "[redacted]")
    return json.loads(serialized)


class ProbeTests(unittest.TestCase):
    revision = "a" * 40
    key = "sb_publishable_TEST_ONLY_NOT_A_REAL_KEY"

    class Response:
        def __init__(self, data, status=200, headers=None):
            self.body = data if isinstance(data, bytes) else json.dumps(data).encode()
            self.status = status
            self.headers = email.message.Message()
            for name, value in (headers or {}).items():
                self.headers[name] = value
        def read(self, size): return self.body[:size]
        def __enter__(self): return self
        def __exit__(self, *args): pass

    def fixture_opener(self, outcome):
        seen = []
        def open_url(request, timeout):
            self.assertEqual(timeout, TIMEOUT)
            self.assertEqual(request.get_method(), "GET")
            seen.append(request)
            if request.full_url == HEAD_URL:
                self.assertIsNone(request.get_header("Apikey"))
                return self.Response({"sha": self.revision})
            if request.full_url.startswith("https://raw.githubusercontent.com/"):
                self.assertIn(self.revision, request.full_url)
                self.assertIsNone(request.get_header("Apikey"))
                source = f'DEFAULT_SUPABASE_URL = {HOST!r}\nDEFAULT_SUPABASE_KEY = {self.key!r}\n'
                return self.Response(source.encode())
            self.assertEqual(urllib.parse.urlparse(request.full_url).hostname, urllib.parse.urlparse(HOST).hostname)
            query = urllib.parse.parse_qs(urllib.parse.urlparse(request.full_url).query)
            self.assertEqual(query["season_id"], ["eq.1601"])
            self.assertEqual(query["limit"], ["1"])
            self.assertEqual(query["select"], [",".join(FIELDS)])
            self.assertEqual(request.get_header("Apikey"), self.key)
            if isinstance(outcome, Exception): raise outcome
            return outcome
        return open_url, seen

    def test_one_row_read_headers_and_count_without_key_disclosure(self):
        sample = {"config_base_id": 123456, "season_id": 1601, "price_fe_median": 12.5}
        for status in (200, 206):
            opener, seen = self.fixture_opener(self.Response([sample], status=status, headers={"Content-Range": "0-0/42", "apikey": self.key}))
            result = probe(opener)
            self.assertEqual(result["server_http_status"], status)
            self.assertEqual(result["total_ids_if_reported"], 42)
            self.assertTrue(result["anonymous_select_confirmed"])
            self.assertTrue(result["ss14_rows_found"])
            self.assertNotIn(self.key, json.dumps(result))
            self.assertEqual(len(seen), 3)  # Two GitHub GETs, exactly one Supabase GET.

    def test_empty_table_is_successful_api_read_not_transport_failure(self):
        opener, _ = self.fixture_opener(self.Response([], headers={"Content-Range": "*/0"}))
        result = probe(opener)
        self.assertTrue(result["anonymous_select_confirmed"])
        self.assertFalse(result["ss14_rows_found"])
        self.assertEqual(result["total_ids_if_reported"], 0)

    def test_proxy_denial_and_timeout_do_not_claim_server_status(self):
        for error in [urllib.error.URLError("Tunnel connection failed: 403 Forbidden"), TimeoutError("timed out")]:
            opener, seen = self.fixture_opener(error)
            result = probe(opener)
            self.assertIsNone(result["server_http_status"])
            self.assertEqual(result["phase"], "supabase")
            self.assertEqual(len(seen), 3)

    def test_http_errors_are_reported_without_retry(self):
        for status in (401, 403, 429, 500, 503):
            headers = email.message.Message(); headers["Retry-After"] = "300"
            opener, seen = self.fixture_opener(urllib.error.HTTPError(HOST, status, "fixture", headers, None))
            result = probe(opener)
            self.assertEqual(result["server_http_status"], status)
            self.assertEqual(result["headers"]["retry-after"], "300")
            self.assertEqual(len(seen), 3)

    def test_malformed_and_oversized_json(self):
        for body in (b"not json", b"x" * (64 * 1024 + 1)):
            opener, _ = self.fixture_opener(self.Response(body))
            self.assertIn("transport_or_format_error", probe(opener))

    def test_wrong_season_cannot_claim_ss14_coverage(self):
        opener, _ = self.fixture_opener(self.Response([{"season_id": 1501, "config_base_id": 123456}]))
        result = probe(opener)
        self.assertFalse(result["ss14_rows_found"])
        self.assertIn("unexpected season", result["transport_or_format_error"])
        self.assertNotIn("total_ids_if_reported", result)

    def test_unexpected_json_shape_and_excess_rows_are_rejected(self):
        for rows in ({"error": "fixture"}, [1], [{"season_id": 1601}, {"season_id": 1601}]):
            opener, _ = self.fixture_opener(self.Response(rows))
            result = probe(opener)
            self.assertIn("transport_or_format_error", result)
            self.assertNotIn("ss14_rows_found", result)

    def test_refuse_redirect_secret_key_and_unexpected_host(self):
        with self.assertRaises(RuntimeError):
            NoRedirect().redirect_request(None, None, 302, "fixture", {}, "https://elsewhere.invalid")
        for host, key in [(HOST, "service_role_not_allowed"), ("https://elsewhere.invalid", self.key)]:
            with self.assertRaises(ValueError):
                public_configuration(f'DEFAULT_SUPABASE_URL={host!r}\nDEFAULT_SUPABASE_KEY={key!r}')


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--output", type=Path, help="Write the same key-free diagnostic JSON to this file")
    parser.add_argument("--self-test", action="store_true", help="Run controlled unit fixtures only, with NO network requests")
    args = parser.parse_args()
    if args.self_test:
        suite = unittest.defaultTestLoader.loadTestsFromTestCase(ProbeTests)
        return 0 if unittest.TextTestRunner(verbosity=2).run(suite).wasSuccessful() else 1
    result = probe()
    text = json.dumps(result, ensure_ascii=False, indent=2)
    if args.output:
        args.output.write_text(text + "\n", encoding="utf-8")
    print(text)
    return 0 if result.get("anonymous_select_confirmed") and not result.get("transport_or_format_error") else 2


if __name__ == "__main__":
    raise SystemExit(main())
