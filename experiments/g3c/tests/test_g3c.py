import contextlib, io, json, os, pathlib, sys, tempfile, unittest
HERE = pathlib.Path(__file__).resolve().parent
sys.path.insert(0, str(HERE.parent / "harness"))
import g3c_harness as H
import g3c_decide as D

F6 = os.environ.get("G3C_F6V2_BIN", "/tmp/envnew/target/release/envmorph-f6v2")


def run(mod, argv):
    buf = io.StringIO()
    with contextlib.redirect_stdout(buf):
        rc = mod.main(argv)
    return rc, buf.getvalue()


class Basics(unittest.TestCase):
    def test_cell_envs(self):
        fac = H.load_factors()
        e = H.build_tool_env(fac, "101", "/w")
        self.assertEqual(e["TZ"], "America/Lima")
        self.assertEqual(e["LC_ALL"], "C")
        self.assertEqual(e["TMPDIR"], "/w/tmp/b")
        self.assertEqual(set(e), {"PATH", "HOME", "TZ", "LC_ALL", "TMPDIR"})
        n = H.build_tool_env(fac, "NULLB", "/w")
        self.assertEqual(n["ENVMORPH_NULL_FACTOR"], "b")
        self.assertEqual(n["TZ"], "UTC")

    def test_guard_without_freeze(self):
        with tempfile.TemporaryDirectory() as t:
            with self.assertRaises(SystemExit) as c, contextlib.redirect_stderr(io.StringIO()):
                H.main(["run-cell", "--workflow", "findutils", "--cell", "100", "--results", t,
                        "--campaign", t, "--work-root", t])
            self.assertEqual(c.exception.code, 3)

    def test_guard_h_requires_seal(self):
        with tempfile.TemporaryDirectory() as t:
            camp = pathlib.Path(t)
            fz = {"harness_sha256": H.sha_file(H.ROOT / "harness" / "g3c_harness.py"),
                  "factors_sha256": H.sha_file(H.ROOT / "factors.json"),
                  "spec_sha256": {"findutils": H.sha_file(H.ROOT / "workflows" / "findutils" / "spec.json")}}
            (camp / "PROTOCOL_FREEZE.json").write_text(json.dumps(fz))
            with self.assertRaises(SystemExit) as c, contextlib.redirect_stderr(io.StringIO()):
                H.main(["run-cell", "--workflow", "findutils", "--cell", "110", "--results", t,
                        "--campaign", t, "--work-root", t])
            self.assertEqual(c.exception.code, 3)

    def test_fixture_digests_are_pinned_and_stable(self):
        fac = H.load_factors()
        for w in ("findutils", "unzip", "exiftool", "patch"):
            spec = H.load_spec(w)
            with tempfile.TemporaryDirectory() as t:
                _, d = H.prepare_workspace(spec, fac, t)
            self.assertEqual(d, spec["fixture_tree_sha256"], w)


@unittest.skipUnless(os.path.isfile(F6), "binario F6-v2 no disponible")
class Pipeline(unittest.TestCase):
    def test_end_to_end_selftest(self):
        wfs = ["_selftest_tz", "_selftest_xor", "_selftest_add", "_selftest_null"]
        with tempfile.TemporaryDirectory() as t:
            res, camp, wr = str(pathlib.Path(t) / "res"), str(pathlib.Path(t) / "camp"), str(pathlib.Path(t) / "w")
            for w in wfs:
                for cell in ("000", "100", "010", "001", "NULLA", "NULLB"):
                    rc, _ = run(H, ["run-cell", "--workflow", w, "--cell", cell, "--results", res,
                                    "--campaign", camp, "--work-root", wr])
                    self.assertEqual(rc, 0)
            # fuga. Una celda H ejecutada antes de decidir debe abortar.
            run(H, ["run-cell", "--workflow", "_selftest_null", "--cell", "111", "--results", res,
                    "--campaign", camp, "--work-root", wr])
            with self.assertRaises(SystemExit) as c, contextlib.redirect_stderr(io.StringIO()):
                D.main(["decide", "--results", res, "--campaign", camp, "--workflows", "_selftest_null", "--f6v2-bin", F6])
            self.assertEqual(c.exception.code, 4)
            for f in (pathlib.Path(res) / "_selftest_null").glob("111_*"):
                f.unlink()
            rc, _ = run(D, ["decide", "--results", res, "--campaign", camp, "--workflows"] + wfs + ["--f6v2-bin", F6])
            self.assertEqual(rc, 0)
            run(D, ["seal", "--results", res, "--campaign", camp])
            for w in wfs:
                for cell in ("110", "101", "011", "111"):
                    rc, _ = run(H, ["run-cell", "--workflow", w, "--cell", cell, "--results", res,
                                    "--campaign", camp, "--work-root", wr])
                    self.assertEqual(rc, 0)
            out = str(pathlib.Path(camp) / "score.json")
            run(D, ["score", "--results", res, "--campaign", camp, "--out", out])
            s = json.loads(pathlib.Path(out).read_text())
            self.assertEqual(s["f6v2_decisiones_erroneas"], 0)
            self.assertTrue(s["workflows"]["_selftest_tz"]["informativo"])
            self.assertFalse(s["workflows"]["_selftest_null"]["informativo"])
            self.assertEqual(s["workflows"]["_selftest_xor"]["celdas"]["110"]["C2"], "wrong")
            self.assertEqual(s["workflows"]["_selftest_xor"]["celdas"]["110"]["C0"], "correct")
            self.assertEqual(s["global"]["C1"]["correct"] + s["global"]["C1"]["wrong"], 0)
            self.assertEqual(s["workflows"]["_selftest_null"]["controles_negativos"], {"NULLA": "STABLE", "NULLB": "STABLE"})


if __name__ == "__main__":
    unittest.main(verbosity=2)
