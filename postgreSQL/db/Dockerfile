# Use official Postgres image as base
FROM postgres:latest

# Copy your initialization scripts into the place Postgres auto-runs them
COPY init/*.sql /docker-entrypoint-initdb.d/

# (Optional) copy any custom config:
# COPY postgres.conf /etc/postgresql/postgresql.conf
# and then add:
# CMD ["postgres", "-c", "config_file=/etc/postgresql/postgresql.conf"]
