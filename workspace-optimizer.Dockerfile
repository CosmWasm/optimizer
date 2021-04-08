FROM cosmwasm/base-optimizer:0.12.0

# Install Python
RUN apk add python3 py3-toml
RUN python3 --version

# Assume we mount the source code in /code
WORKDIR /code

# Add our scripts as entry point
ADD build_workspace.py /usr/local/bin/
ADD optimize_workspace.sh /usr/local/bin/
RUN chmod +x /usr/local/bin/build_workspace.py
RUN chmod +x /usr/local/bin/optimize_workspace.sh

ENTRYPOINT ["optimize_workspace.sh"]
# Default argument when none is provided
CMD ["."]
