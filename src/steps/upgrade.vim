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
    if $TOPGRADE_FORCE_PLUGUPDATE
        PlugUpdate!
    else
        PlugUpdate
    endif
endif

if exists(":PackerUpdate")
    echo "Packer"
    PackerSync
endif

if exists("*dein#update()")
    echo "dein#update()"
    call dein#update()
endif

if exists(":DeinUpdate")
    echo "DeinUpdate"
    DeinUpdate
endif

if exists(":PaqUpdate")
    echo "PaqUpdate"
    PaqUpdate
endif

if exists(":CocUpdateSync")
    echo "CocUpdateSync"
    CocUpdateSync
endif

quitall
