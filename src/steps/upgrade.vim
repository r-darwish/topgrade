if exists(":NeoBundleUpdate")
    echo "NeoBundle"
    NeoBundleUpdate
endif

if exists(":PluginUpdate")
    echo "Plugin"
    PluginUpdate
endif

if exists(":PlugUpgrade")
    echo "Plug"
    PlugUpgrade
    PlugUpdate
endif

if exists(":PackerUpdate")
    echo "Packer"
    PackerSync
endif

if exists(":DeinUpdate")
    echo "DeinUpdate"
    DeinUpdate
endif

quitall
