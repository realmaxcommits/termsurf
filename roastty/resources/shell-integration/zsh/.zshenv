# Based on (started as) a copy of Kitty's zsh integration. Kitty is
# distributed under GPLv3, so this file is also distributed under GPLv3.
# The license header is reproduced below:
#
# This program is free software: you can redistribute it and/or modify
# it under the terms of the GNU General Public License as published by
# the Free Software Foundation, either version 3 of the License, or
# (at your option) any later version.
#
# This program is distributed in the hope that it will be useful,
# but WITHOUT ANY WARRANTY; without even the implied warranty of
# MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
# GNU General Public License for more details.
#
# You should have received a copy of the GNU General Public License
# along with this program.  If not, see <http://www.gnu.org/licenses/>.

# This script is sourced automatically by zsh when ZDOTDIR is set to this
# directory. It therefore assumes it's running within our shell integration
# environment and should not be sourced manually (unlike roastty-integration).
#
# This file can get sourced with aliases enabled. To avoid alias expansion
# we quote everything that can be quoted. Some aliases will still break us
# though.

# Restore the original ZDOTDIR value if ROASTTY_ZSH_ZDOTDIR is set.
# Otherwise, unset the ZDOTDIR that was set during shell injection.
if [[ -n "${ROASTTY_ZSH_ZDOTDIR+X}" ]]; then
    'builtin' 'export' ZDOTDIR="$ROASTTY_ZSH_ZDOTDIR"
    'builtin' 'unset' 'ROASTTY_ZSH_ZDOTDIR'
else
    'builtin' 'unset' 'ZDOTDIR'
fi

# Use try-always to have the right error code.
{
    # Zsh treats unset ZDOTDIR as if it was HOME. We do the same.
    #
    # Source the user's .zshenv before sourcing roastty-integration because the
    # former might set fpath and other things without which roastty-integration
    # won't work.
    #
    # Use typeset in case we are in a function with warn_create_global in
    # effect. Unlikely but better safe than sorry.
    'builtin' 'typeset' _roastty_file=${ZDOTDIR-$HOME}"/.zshenv"
    # Zsh ignores unreadable rc files. We do the same.
    # Zsh ignores rc files that are directories, and so does source.
    [[ ! -r "$_roastty_file" ]] || 'builtin' 'source' '--' "$_roastty_file"
} always {
    if [[ -o 'interactive' ]]; then
        # ${(%):-%x} is the path to the current file.
        # On top of it we add :A:h to get the directory.
        'builtin' 'typeset' _roastty_file="${${(%):-%x}:A:h}"/roastty-integration
        if [[ -r "$_roastty_file" ]]; then
            'builtin' 'autoload' '-Uz' '--' "$_roastty_file"
            "${_roastty_file:t}"
            'builtin' 'unfunction' '--' "${_roastty_file:t}"
        fi
    fi
    'builtin' 'unset' '_roastty_file'
}
