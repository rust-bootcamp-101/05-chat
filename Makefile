.PHONY: chat_server
chat_server:
	@cargo run --bin chat-server

.PHONY: notify_server
notify_server:
	@cargo run --bin notify-server
