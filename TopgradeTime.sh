$FINISHED = false

echo Enter the hours you would like to run topgrade or enter never to have the choice to run it manually.
read $HOURS

if [$HOURS = "never"]
then
  0 0 2 31 0 topgrade # Never run topgrade as 31/2 does not exist.

echo Enter the minuites of the time you would like to run it.
read $MINS

$MINS $HOURS * * * topgrade ## Run at the specified time
