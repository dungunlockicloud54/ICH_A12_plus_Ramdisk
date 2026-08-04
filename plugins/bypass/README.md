# Plugin skeleton for bypass modules

Place custom bypass scripts here. Each bypass plugin should be an executable script with the following contract:

- Filename should be the plugin id, e.g. bypass_method_x.sh
- The script will be invoked with two arguments: UDID and path to a temp output file.
- The script should write progress logs to stdout and a final JSON result to the provided output file.
- Exit code 0 indicates success, non-zero indicates failure.

Example call from backend (not implemented yet):
  ./plugins/bypass/bypass_method_x.sh <UDID> /tmp/bypass_out.json

Security: the GUI will not run bypass plugins automatically; they must be selected by the user from the Settings -> Bypass Plugins list.
