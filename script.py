import os
import subprocess
import re
import tempfile
import json


jsons = {} 

for (dirpath, dirnames, filenames) in os.walk('./autocomplete/src/'): 
    for file in filenames:
        finalpath = ""
        if dirpath[-1] != '/':  
            finalpath = dirpath + '/' + file  
        else:  
            finalpath = dirpath + file 
 
        print(finalpath) 
        text = "" 
        try: 
            text = open(finalpath, 'r').read() 
        except: 
            pass 
         
        if len(text) == 0: continue 
        finaltext = re.sub(r'^[^{]*{', '{', text) 
        finaltext = "".join(reversed(re.sub(r'^[^}]*}', '}', "".join(reversed(finaltext))))) 
        tempFile = open('temp.js', 'w') 
        tempFile.write('console.log(JSON.stringify(' + finaltext + '))') 
        tempFile.close() 
        finalThing = "" 
        try: 
            finalThing = subprocess.run(['node', 'temp.js'], capture_output=True).stdout 
        except Exception as e: 
            print("failure\n" + finaltext + '\nerr\n' + e.message) 
         
        try:  
            finalThing = json.loads(finalThing) 
            print(finalpath + ' works') 
            pfx = ""

            if dirpath[-1] != '/':
                pfx = 'defs/'+ dirpath.split('/')[-1] + '-'
            else:
                pfx = 'defs/'
            file = open(pfx + finalThing["name"] + '.json', 'w')
            json.dump(finalThing, file)
            file.close()
        except: 
            print(finalpath + " didn't work")
