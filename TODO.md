# TODO

- [ ] better error handler
- [ ] log rotate
- [ ] support validate config before save(web)
- [ ] http response cache(storage: tinyufo, file)
- [ ] support validate config for plugin
- [ ] secret storage
- [ ] different servers avoid listening to the same address
- [ ] support include comnand for configuraion
- [ ] upstream_response_time
- [x] support set_cipher_list and set_ciphersuites
- [x] support setting max ttl of cache
- [x] support set header for static serve
- [x] set header support `remote_addr`
- [x] hot reload upstream and location config
- [x] jwt sign response plugin(sign path)
- [x] jwt auth ignore path
- [x] tcp fast open for upstream
- [x] support keepalive for server accept connection
- [x] jwt auth proxy plugin
- [x] location supports client body size limit
- [x] csrf plugin
- [x] referer restriction plugin
- [x] upstream select algo support more options
- [x] tls validity checker
- [x] fix not_before not_after of cert
- [x] http header convert $HOSTNAME for hostname
- [x] http headers plugin
- [x] support `Content-Disposition` for directory static serve plugin
- [x] support more limit plugin
- [x] how to use proxy plugin
- [x] server listen multi address
- [x] show name of web view editor
- [x] add more digest from session.digest
- [x] toml config preview
- [x] support ttl limit
- [x] disable notification event
- [x] use buffer writer for log
- [x] convert plugin category to string?
- [x] support i18n
- [x] access log format performance
- [x] support etcd or other storage for config
- [x] send more event to webhook
- [x] remove config
- [x] diff config and send to webhook
- [x] support pyroscope
- [x] tls cert auto update[instant-acme]
- [x] delay restart
- [x] redirect http to https(orginal uri)
- [x] headers for location support get from env
- [x] basic auth
- [x] allow none upstream for location
- [x] allow deny ip proxy plugin
- [x] auto reload config and restart
- [x] request id proxy plugin
- [x] support plugin for proxy and response
- [x] authentication for admin page
- [x] custom error for pingora error
- [x] support alpn for location
- [x] support add header for location
- [x] support x-forwarded-for
- [x] error page
- [x] http peer option
- [x] access log
- [x] support format for env logger(or tokio tracing)
- [x] config validate
- [x] support add tls
- [x] stats of server
- [x] start without config
- [x] static serve for admin
- [x] status:499 for client abort
- [x] support get pingap start time
- [x] static file serve
- [x] set priority for location
- [x] mock response for upstream
- [x] add remark for config
- [x] support multi host for location?
- [x] support set upstream_keepalive_pool_size
- [x] graceful restart for admin web
- [x] use stable pingora
- [x] support web hook for backend health check
- [x] sentry uri config
- [x] charset for static file
- [x] web hook for wecom, dingtalk robot
- [x] verify_cert option for http peer
- [x] compression: zstd, br, gzip
- [x] support set threads for each server
- [x] location limit of request: ip or custom field
