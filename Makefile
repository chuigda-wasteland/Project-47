.PHONY: miri_test_pr47_core_data miri_test_pr47_core_data_prompt

miri_test_pr47_core_data:
	@echo testing pr47::core::data
	@cargo +nightly miri test --package pr47 --lib data::test
