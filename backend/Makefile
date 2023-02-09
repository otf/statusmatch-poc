.PHONY: public
public:
	cp -r ../frontend/public ./

.PHONY: server
server: public
	cargo shuttle run
	

.PHONY: deploy
deploy: public
	git add public/ && git commit -m 'add public to deploy static.'
	-cargo shuttle deploy
	git reset --hard HEAD^

.PHONY: clean
clean:
	rm -rf public