use crate::constants;

pub fn show_version() {
    print!(
        "{} version {}
Copyright (C) 2022-2023 by Andreas Maus <maus@ypbind.de>
This program comes with ABSOLUTELY NO WARRANTY.

{} is distributed under the Terms of the GNU General
Public License Version 3. (http://www.gnu.org/copyleft/gpl.html)

",
        constants::PACKAGE_NAME,
        constants::VERSION,
        constants::PACKAGE_NAME
    );
}
