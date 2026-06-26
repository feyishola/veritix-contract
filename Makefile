.PHONY: e2e-test

e2e-test:
	@echo "Running end-to-end event lifecycle test..."
	@bash scripts/e2e_event_lifecycle.sh