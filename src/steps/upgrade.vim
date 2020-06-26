PlugUpdate1
echo "one"
if exists(":NeoBundleUpdate")
echo "two"
    NeoBundleUpdate
endif

echo "asd"
if exists(":PluginUpdate")
    PluginUpdate
endif

echo "hq"
if exists(":PlugUpgrade")
    echo "Plug"
    PlugUpgrade
    PlugClean
    PlugUpdate
endif
