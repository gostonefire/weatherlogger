# Configure Systemd
* Check paths in `start.sh` and `weatherlogger.service`
* Copy `weatherlogger.service` to `/lib/systemd/system/`
* Run `sudo systemctl enable weatherlogger.service`
* Run `sudo systemctl start weatherlogger.service`
* Check status by running `sudo systemctl status weatherlogger.service`

Output should be something like:
```
● weatherlogger.service - Weatherlogger for Mygrid Dash
     Loaded: loaded (/lib/systemd/system/weatherlogger.service; enabled; preset: enabled)
     Active: active (running) since Fri 2025-07-25 13:54:00 CEST; 9s ago
   Main PID: 95159 (bash)
      Tasks: 8 (limit: 9573)
        CPU: 193ms
     CGroup: /system.slice/weatherlogger.service
             ├─95159 /bin/bash /home/petste/MyWeatherLogger/start.sh
             └─95160 /home/petste/MyWeatherLogger/weatherlogger --config=/home/petste/MyWeatherLogger/config/config.toml

Jul 25 13:54:00 mygrid systemd[1]: Started weatherlogger.service - Weatherlogger for Mygrid Dash.
```

If the application for some reason prints anything to stdout/stderr, such in case of a panic,
the log for that can be found by using `journalctl -u weatherlogger.service`.