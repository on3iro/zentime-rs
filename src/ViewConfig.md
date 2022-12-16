# Type of client interface

Supports:

  * default - TUI interface including keyboard shortcuts
  * minimal - minimal colored output
  * raw - continous stream of uncolored printed lines - is not able to take keyboard input
  and therefore needs another client to start/stop the timer. This is useful for display
  integration into tools like tmux where no interaction is necessary.
