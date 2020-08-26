(if (fboundp 'paradox-upgrade-packages)
    (progn
      (unless (boundp 'paradox-github-token)
        (setq paradox-github-token t))
      (paradox-upgrade-packages)
      (princ
       (if (get-buffer "*Paradox Report*")
           (with-current-buffer "*Paradox Report*" (buffer-string))
         "\nNothing to upgrade\n")))
  (progn
    (let ((package-menu-async nil))
      (package-list-packages))
    (package-menu-mark-upgrades)
    (condition-case nil
        (package-menu-execute 'noquery)
      (user-error nil))))
