.PHONY: initrepo
initrepo:
	@pre-commit install
	@cargo deny fetch

.PHONY: genkey
genkey:
	@openssl genpkey -algorithm ed25519 -out encoding.pem
	@openssl pkey -in encoding.pem -pubout -out decoding.pem
