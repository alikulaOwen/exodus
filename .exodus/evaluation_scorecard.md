# Project Exodus Benchmark Scorecard

**Total Fixtures**: 11 | **Exodus Behavioral Pass Rate**: 90.9% (Baseline: 27.3%)

**Compilation Rate**: 100.0% (Baseline: 36.4%) | **Migration Debts**: 1 | **Human Interventions**: 0

| Fixture | Symbols | Exodus Outcome | Behavioral Tests | Debts | Human Review | Baseline Pass |
|---|---|---|---|---|---|---|
| `01_typed_functions` | 3 | `Verified` | ✅ PASS | 0 | None | ✅ PASS |
| `02_class_conversion` | 1 | `Verified` | ✅ PASS | 0 | None | ✅ PASS |
| `03_module_dependency` | 2 | `Verified` | ✅ PASS | 0 | None | ✅ PASS |
| `04_circular_dependency` | 2 | `Verified` | ✅ PASS | 0 | None | ❌ FAIL |
| `05_unsupported_decorator` | 2 | `Verified` | ✅ PASS | 0 | None | ❌ FAIL |
| `06_dynamic_value` | 1 | `Verified` | ✅ PASS | 0 | None | ❌ FAIL |
| `07_missing_sdk` | 1 | `Verified` | ✅ PASS | 0 | None | ❌ FAIL |
| `08_async_function` | 1 | `Verified` | ✅ PASS | 0 | None | ❌ FAIL |
| `09_database_compat_wrapper` | 1 | `Verified` | ✅ PASS | 0 | None | ❌ FAIL |
| `10_deliberately_untranslatable_reflection` | 2 | `Degraded` | ❌ FAIL | 1 | None | ❌ FAIL |
| `two_run_demo` | 2 | `Verified` | ✅ PASS | 0 | None | ❌ FAIL |
