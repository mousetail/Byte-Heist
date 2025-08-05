import re
import requests
import os

font_regex = f"^\\s+src: url\\((https://fonts\\.gstatic\\.com/s/\\w+/(?:\\w|/)+\\.woff2)\\)"

if __name__=="__main__":
    mp = {}
    mp2 = {}

    with open("fonts.css", "r") as source, open("font_output.css", "w") as dest:
        font_name = ""
        font_script = ""
        for line in source:
            if e:=re.match(r"^/\* ((?:\w|-)+) \*/", line):
                font_script = e[1]
            elif e:=re.match(f"^\\s+font-family: \"((?:\\w| )+)\";", line):
                font_name = e[1]
            elif e:=re.match(font_regex, line):
                font_url = e[1]

                if font_url not in mp:
                    mp[font_url] = {"font_family": font_name, "scripts": set()}
                mp[font_url]["scripts"].add(font_script)

        source.seek(0)

        for key, value in mp.items():
            font_file_name = value["font_family"].replace(" ","")+"_"+"_".join(value["scripts"])+".woff2"

            if not os.path.exists(font_file_name):
                res = requests.get(key)
                with open(font_file_name, "wb") as f:
                    f.write(res.content)

            mp2[key] = font_file_name

        for line in source:
            dest.write(re.sub(font_regex, lambda m:f"  src: url('./{mp2[m[1]]}')", line))

