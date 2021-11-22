pear() {
	zle -I
	echo $BUFFER
}
zle -N pear

bindkey '^o' pear
