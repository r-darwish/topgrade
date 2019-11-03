(cond
 ((fboundp 'paradox-upgrade-packages)
  (princ "Upgrading packages with Paradox")
  (paradox-upgrade-packages)
  (princ
   (if (get-buffer "*Paradox Report*")
       (with-current-buffer "*Paradox Report*" (buffer-string))
     "\nNothing to upgrade\n")))
 ((fboundp 'straight-thaw-versions)
  (princ "Thawing versions")
  (straight-thaw-versions))
 (t
  (princ "Upgrading packages")
  (let ((package-menu-async nil))
    (package-list-packages))
  (package-menu-mark-upgrades)
  (condition-case nil
      (package-menu-execute 'noquery)
    (user-error nil))))