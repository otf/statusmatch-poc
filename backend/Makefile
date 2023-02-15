.PHONY: public
public:
	cp -r ../frontend/public ./

.PHONY: server
server: public
	cargo shuttle run
	

.PHONY: deploy
deploy: public
	cargo shuttle deploy

.PHONY: clean
clean:
	rm -rf public
