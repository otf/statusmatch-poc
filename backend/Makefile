.PHONY: public
public:
	cp -r ../frontend/public ./

.PHONY: server
server: public
	cargo run

.PHONY: clean
clean:
	rm -rf public
