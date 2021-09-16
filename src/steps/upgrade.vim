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
    augroup TOPGRADE_AUCMDS
      au!
      autocmd User PackerComplete quitall
    augroup END
    PackerSync
    finish
endif

if exists(":DeinUpdate")
    echo "DeinUpdate"
    DeinUpdate
endif

if exists(":PaqUpdate")
    echo "PaqUpdate"
    PaqUpdate
endif

quitall
