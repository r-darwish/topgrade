if exists(":NeoBundleUpdate")
    echo NeoBundle
    NeoBundleUpdate
endif

if exists(":PluginUpdate")
    echo "Plugin"
    PluginUpdate
endif

if exists(":PlugUpgrade")
    echo "Plug"
    PlugUpgrade
    PlugClean
    PlugUpdate
endif

quitall