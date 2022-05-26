(when (fboundp 'paradox-upgrade-packages)
    (progn
      (unless (boundp 'paradox-github-token)
        (setq paradox-github-token t))
      (paradox-upgrade-packages)
      (princ
       (if (get-buffer "*Paradox Report*")
           (with-current-buffer "*Paradox Report*" (buffer-string))
         "\nNothing to upgrade\n"))))
