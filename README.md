# Log Analyzer Pro (lap)
A powerful log analyzer application for the terminal written in Rust

![demo](media/demo.gif)


## Features
* Read logs from files or sockets. It updates live with new entries
* Optionally format logs with a regex expression to match any of the Date, Timestamp, App, Severity, Function, Payload categories and ease reading and filtering
* Include, exclude or marker regex filters
* Regex search

## Installation

````
cargo install log-analyzer-pro
````

The binary executable is `lap`


## Usage
### Menu navigation
Use <kbd>Shift</kbd> + <kbd>Arrows</kbd> or <kbd>⇥ Tab</kbd> to navigate across the application menus and change focus.
* Left: <kbd>⇧ Shift</kbd> + <kbd>←</kbd>
* Right: <kbd>⇧ Shift</kbd> + <kbd>→</kbd>
* Up: <kbd>⇧ Shift</kbd> + <kbd>↑</kbd>
* Down: <kbd>⇧ Shift</kbd> + <kbd>↓</kbd>

### Inner navigation
When focused on a tab, list or table, use the <kbd>Arrows</kbd> to change the selection
* Left: <kbd>⇧ Shift</kbd> + <kbd>←</kbd>
* Right: <kbd>⇧ Shift</kbd> + <kbd>→</kbd>
* Up: <kbd>⇧ Shift</kbd> + <kbd>↑</kbd>
* Down: <kbd>⇧ Shift</kbd> + <kbd>↓</kbd>

### Inner navigation
When focused on a tab, list or table, use the <kbd>Arrows</kbd> to change the selection
* Left: <kbd>←</kbd>
* Right: <kbd>→</kbd>
* Up: <kbd>↑</kbd>
* Down: <kbd>↓</kbd>

### Resize modules
* Resize Left: <kbd>⇧ Shift</kbd> + <kbd>A</kbd>
* Resize Right: <kbd>⇧ Shift</kbd> + <kbd>D</kbd>
* Resize Up: <kbd>⇧ Shift</kbd> + <kbd>W</kbd>
* Resize Down: <kbd>⇧ Shift</kbd> + <kbd>S</kbd>


### Sources Module
* <kbd>+</kbd> or <kbd>i</kbd> to add new log

### Filters Module
* Add new filter: <kbd>+</kbd> or <kbd>i</kbd> to
* Use `inner navigation` to select a filter
* Edit selected filter: <kbd>e</kbd>

### Log & Search results module
*
* Use `inner navigation` to navigate through the logs and apply horizontal scroll
* Press <kbd>⌥ Option</kbd> or <kbd>Alt</kbd> + `inner navigation` for rapid scroll
* Press <kbd>Page Up</kbd> or <kbd>Page Down</kbd> to paginate 1000 lines
* Navigate to index (or closest): <kbd>⇧ Shift</kbd> + <kbd>G</kbd>
* Toggle columns ON/OFF:
    - <kbd>i</kbd>: Index
    - <kbd>d</kbd>: Date
    - <kbd>d</kbd>: Timestamp
    - <kbd>a</kbd>: App
    - <kbd>s</kbd>: Severity
    - <kbd>f</kbd>: Function
    - <kbd>p</kbd>: Payload


* If you're in `Search results` you can go to the selected index in `Log module`: <kbd>⏎ Enter</kbd>

### Search highlighting
You can highlight search results by using regex groups in your search. The name of the group should be the color you want to highlight the match with. The list of available colors is:
- BLACK
- WHITE
- RED
- GREEN
- YELLOW
- BLUE
- MAGENTA
- CYAN
- GRAY
- DARKGRAY
- LIGHTRED
- LIGHTGREEN
- LIGHTYELLOW
- LIGHTBLUE
- LIGHTMAGENTA
- LIGHTCYAN

Search example:
```
(?P<GREEN>success_ok).*(?P<BLUE>message)
````

## Customization
You can use a json file to customize the application look and preload formats and filters by using a command line argument:

````
lap --settings path_to_settings_file.json
````

* Primary color: RGB tuple (reed, green, blue)
* Formats: List of {alias, regex}
    - The regex is used to format lines into the available columns. To do so you need to capture groups. The valid groups are:
        - DATE
        - TIMESTAMP
        - APP
        - SEVERITY
        - FUNCTION
        - PAYLOAD
* Filters: List of {alias, action, filter}
    - action: One of `{INCLUDE, EXCLUDE, MARKER}`
    - filter: Dictionary of `{column_name: regex and color: RGB tuple (reed, green, blue)}`. All fields are optional

Example file
```json
{
    "primary_color": [0, 225, 255],
    "formats": [
        {
            "alias": "Default",
            "regex": "(?P<PAYLOAD>.*)"
        },
        {
            "alias": "Application",
            "regex": "(?P<DATE>[\\d]{4}-[\\d]{2}-[\\d]{2} [\\d]{2}:[\\d]{2}:[\\d]{2},[\\d]{3}) - \\[(?P<SEVERITY>[\\w]*)\\] - \\[([ \\w]{4})\\] - \\[(?P<TIMESTAMP>[ \\d]*)\\] (?P<PAYLOAD>.*)"
        },
        {
            "alias": "System",
            "regex": "(?P<DATE>[\\d]{4}-[\\d]{2}-[\\d]{2} [\\d]{2}:[\\d]{2}:[\\d]{2}.[\\d]*) \\((?P<APP>[\\w\\d]*)[/ ]?(?P<FUNCTION>.*)\\) (?P<PAYLOAD>.*)"
        }
    ],
    "filters": [
        {
            "alias": "System",
            "action": "MARKER",
            "filter": {
                "app": "system",
                "color": [100, 100, 0]
            }
        },
        {
            "alias": "SIGKILL",
            "action": "MARKER",
            "filter": {
                "payload": "SIGKILL",
                "color": [255, 0, 0]
            }
        }
    ]
}
```

## License
Dual-licensed under MIT or the [UNLICENSE](https://unlicense.org).