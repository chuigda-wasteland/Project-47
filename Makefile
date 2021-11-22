.PHONY: all
all:
	@echo "Please, run something else"

ASYNC_BENCH_FEATURES = --no-default-features --features="async async-avoid-block async-tokio bench optimized-rtlc"
SYNC_BENCH_FEATURES = --no-default-features --features="bench optimized-rtlc"

.PHONY:
clean:
	@echo "Nothing to be done for clean operation"

.PHONY: run_brute_force_proof
run_brute_force_proof:
	@rustc brute_force_proof.rs -C opt-level=2
	@./brute_force_proof show-error show-unreachable

.PHONY: test_sync
test_sync:
	cargo test ${SYNC_BENCH_FEATURES}
	cargo test --release ${SYNC_BENCH_FEATURES}

.PHONY: test_async
test_async:
	cargo test ${ASYNC_BENCN_FEATURES}
	cargo test --release ${ASYNC_BENCN_FEATURES}

.PHONY: bench_al31fm2_fib35
bench_al31fm2_fib35:
	@BENCH_ITEM="fib35" cargo run --release ${ASYNC_BENCH_FEATURES}  --bin bench_al31fm2

.PHONY: bench_al31fm2_new1m
bench_al31fm2_new1m:
	@BENCH_ITEM="new1m" cargo run --release ${ASYNC_BENCH_FEATURES} --bin bench_al31fm2

.PHONY: bench_al31fm2_raw_iter
bench_al31fm2_raw_iter:
	@BENCH_ITEM="raw_iter" cargo run --release ${ASYNC_BENCH_FEATURES} --bin bench_al31fm2

.PHONY: bench_al31fm2_ffi
bench_al31fm2_ffi:
	@BENCH_ITEM="ffi" cargo run --release ${ASYNC_BENCH_FEATURES} --bin bench_al31fm2

.PHONY: bench_al31fm2_sync_fib35
bench_al31fm2_sync_fib35:
	@BENCH_ITEM="fib35" cargo run --release ${SYNC_BENCH_FEATURES} --bin bench_al31fm2_sync

.PHONY: bench_al31fm2_sync_new1m
bench_al31fm2_sync_new1m:
	@BENCH_ITEM="new1m" cargo run --release ${SYNC_BENCH_FEATURES} --bin bench_al31fm2_sync

.PHONY: bench_al31fm2_sync_raw_iter
bench_al31fm2_sync_raw_iter:
	@BENCH_ITEM="raw_iter" cargo run --release ${SYNC_BENCH_FEATURES} --bin bench_al31fm2_sync

.PHONY: bench_al31fm2_sync_ffi
bench_al31fm2_sync_ffi:
	@BENCH_ITEM="ffi" cargo run --release ${SYNC_BENCH_FEATURES} --bin bench_al31fm2_sync

.PHONY: flamegraph_bench_al31fm2_fib35
flamegraph_bench_al31fm2_fib35:
	@BENCH_ITEM="fib35" cargo flamegraph ${ASYNC_BENCH_FEATURES} --bin bench_al31fm2

.PHONY: flamegraph_bench_al31fm2_new1m
flamegraph_bench_al31fm2_new1m:
	@BENCH_ITEM="new1m" cargo flamegraph ${ASYNC_BENCH_FEATURES}--bin bench_al31fm2

.PHONY: flamegraph_bench_al31fm2_raw_iter
flamegraph_bench_al31fm2_raw_iter:
	@BENCH_ITEM="raw_iter" cargo flamegraph ${ASYNC_BENCH_FEATURES} --bin bench_al31fm2

.PHONY: flamegraph_bench_al31fm2_ffi
flamegraph_bench_al31fm2_ffi:
	@BENCH_ITEM="ffi" cargo flamegraph ${ASYNC_BENCH_FEATURES} --bin bench_al31fm2

.PHONY: flamegraph_bench_al31fm2_sync_ffi
flamegraph_bench_al31fm2_sync_ffi:
	@BENCH_ITEM="ffi" cargo flamegraph ${SYNC_BENCH_FEATURES} --bin bench_al31fm2_sync

.PHONY: flamegraph_bench_al31fm2_sync_fib35
flamegraph_bench_al31fm2_sync_fib35:
	@BENCH_ITEM="fib35" cargo flamegraph ${SYNC_BENCH_FEATURES} --bin bench_al31fm2_sync

.PHONY: flamegraph_bench_al31fm2_sync_raw_iter
flamegraph_bench_al31fm2_sync_raw_iter:
	@BENCH_ITEM="raw_iter" cargo flamegraph ${SYNC_BENCH_FEATURES} --bin bench_al31fm2_sync

.PHONY: flamegraph_bench_al31fm2_sync_new1m
flamegraph_bench_al31fm2_sync_new1m:
	@BENCH_ITEM="new1m" cargo flamegraph ${SYNC_BENCH_FEATURES} --bin bench_al31fm2_sync

.PHONY: miri_test_pr47_core_data
miri_test_pr47_core_data:
	@echo testing pr47::core::data
	@cargo +nightly miri test --package pr47 --lib data::test

.PHONY: miri_test_pr47_tyck_pool
miri_test_pr47_tyck_pool:
	@echo testing pr47::core::data::tyck::TyckPool
	@cargo +nightly miri test --package pr47 --lib data::tyck::test_tyck_info_pool::test_tyck_info_pool

.PHONY: miri_test_pr47_core_util_serializer
miri_test_pr47_core_util_serializer:
	@MIRIFLAGS="-Zmiri-disable-isolation" cargo +nightly miri test --package pr47 --lib util::serializer

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

.PHONY: miri_test_pr47_vm_al31f
miri_test_pr47_vm_al31f:
	@MIRIFLAGS="-Zmiri-disable-isolation" cargo +nightly miri test --package pr47 --lib vm::al31f::test

.PHONY: miri_test_pr47_vm_al31f_release
miri_test_pr47_vm_al31f_release:
	@MIRIFLAGS="-Zmiri-disable-isolation" cargo +nightly miri test --release --package pr47 --lib vm::al31f::test
