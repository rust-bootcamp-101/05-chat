.PHONY: initrepo
initrepo:
	@pre-commit install
	@cargo deny fetch
