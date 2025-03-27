# файл webview_test.py
import gi
gi.require_version('Gtk', '3.0')
gi.require_version('WebKit2', '4.1')
from gi.repository import Gtk, WebKit2

win = Gtk.Window()
win.set_default_size(800, 600)
webview = WebKit2.WebView()
webview.load_uri("https://example.com")
win.add(webview)
win.connect("destroy", Gtk.main_quit)
win.show_all()
Gtk.main()
