export EVCXR_CONFIG_DIR=$(CURDIR)/evcxr

help: ## print this message
	@echo "Command list:"
	@echo ""
	@printf "\033[36m%-30s\033[0m %-50s %s\n" "[Sub command]" "[Description]" "[Example]"
	@grep -E '^[/a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | perl -pe 's%^([/a-zA-Z_-]+):.*?(##)%$$1 $$2%' | awk -F " *?## *?" '{printf "\033[36m%-30s\033[0m %-50s %s\n", $$1, $$2, $$3}'

.PHONY: run
run: ## console版nisshiees-coffeeを実行
	evcxr

.PHONY: dep
dep: ## console版nisshiees-coffeeを実行するための依存ツールをインストール
	cargo install -f evcxr_repl
