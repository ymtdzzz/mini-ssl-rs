FROM nginx:1.17

# COPY nginx.conf /etc/nginx
# COPY default.conf /etc/nginx/conf.d
COPY gen_htpasswd /etc/nginx

RUN apt-get update
RUN apt-get install -y openssl

RUN /etc/nginx/gen_htpasswd
