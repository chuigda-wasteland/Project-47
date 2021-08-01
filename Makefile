.PHONY: flamegraph_bench_al31fm2
flamegraph_bench_al31fm2:
	@cargo flamegraph --bench bench_al31fm2 --features=bench -- --nocapture

.PHONY: miri_test_pr47_core_data
miri_test_pr47_core_data:
	@echo testing pr47::core::data
	@cargo +nightly miri test --package pr47 --lib data::test

.PHONY: miri_test_pr47_tyck_pool
miri_test_pr47_tyck_pool:
	@echo testing pr47::core::data::tyck::TyckPool
	@cargo +nightly miri test --package pr47 --lib data::tyck::test_tyck_info_pool::test_tyck_info_pool

.PHONY: miri_test_pr47_core_data_dyn_base_assoc
miri_test_pr47_core_data_dyn_base_assoc:
	@cargo +nightly miri test --package pr47 --lib data::test::test_dyn_base_assoc

.PHONY: miri_test_pr47_core_data_value_assoc_container
miri_test_pr47_core_data_value_assoc_container:
	@cargo +nightly miri test --package pr47 --lib data::test::test_value_assoc_container

.PHONY: miri_test_pr47_core_data_value_assoc_custom_container
miri_test_pr47_core_data_value_assoc_custom_container:
	@cargo +nightly miri test --package pr47 --lib data::test::test_value_assoc_custom_container

.PHONY: miri_test_pr47_core_al31f_default_alloc_simple
miri_test_pr47_core_al31f_default_alloc_simple:
	@cargo +nightly miri test --package pr47 --lib vm::al31f::alloc::default_alloc::test::test_default_collector_simple

.PHONY: miri_test_pr47_core_al31f_default_alloc_custom_vt
miri_test_pr47_core_al31f_default_alloc_custom_vt:
	@cargo +nightly miri test --package pr47 --lib vm::al31f::alloc::default_alloc::test::test_default_collector_custom_vt