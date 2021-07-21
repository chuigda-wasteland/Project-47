.PHONY: miri_test_pr47_core_data
miri_test_pr47_core_data:
	@echo testing pr47::core::data
	@cargo +nightly miri test --package pr47 --lib data::test

.PHONY: miri_test_pr47_tyck_pool
miri_test_pr47_tyck_pool:
	@echo testing pr47::core::data::tyck::TyckPool
	@cargo +nightly miri test --package pr47 --lib data::tyck::test_tyck_info_pool::test_tyck_info_pool
