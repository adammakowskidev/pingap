import useConfigState from "@/states/config";
import { formatError } from "@/helpers/util";
import { useAsync } from "react-async-hook";
import { useToast } from "@/hooks/use-toast";
import { useTranslation } from "react-i18next";
import { Card } from "@/components/ui/card";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs";
import { useShallow } from "zustand/react/shallow";
import { ClipboardCopy } from "lucide-react";
import { Button } from "@/components/ui/button";
import { cn } from "@/lib/utils";
import { Textarea } from "@/components/ui/textarea";
import React from "react";

export default function Config() {
  const { t } = useTranslation();
  const { toast } = useToast();
  const [importing, setImportings] = React.useState(false);
  const [newToml, setNewToml] = React.useState("");
  const [fetchToml, importToml, fullToml, originalToml] = useConfigState(
    useShallow((state) => [
      state.fetchToml,
      state.importToml,
      state.fullToml,
      state.originalToml,
    ]),
  );
  useAsync(async () => {
    try {
      await fetchToml();
    } catch (err) {
      toast({
        title: t("fetchFail"),
        description: formatError(err),
      });
    }
  }, []);

  const copyToml = async () => {
    try {
      await navigator.clipboard.writeText(originalToml);
      toast({
        title: t("copyTomlSuccess"),
      });
    } catch (err) {
      toast({
        title: t("copyTomlFail"),
        description: formatError(err),
      });
    }
  };
  const handleImportToml = async (value: string) => {
    if (importing) {
      return;
    }
    setImportings(true);
    try {
      await importToml(value);
      toast({
        title: t("importSuccess"),
      });
    } catch (err) {
      toast({
        title: t("importFail"),
        description: formatError(err),
      });
    } finally {
      setImportings(false);
    }
  };

  const different = fullToml != originalToml;
  let tabClass = "grid-cols-2";
  if (different) {
    tabClass = "grid-cols-3";
  }
  let importText = t("import");
  if (importing) {
    importText += "...";
  }
  return (
    <div className="grow lg:border-l overflow-auto p-4">
      <Tabs defaultValue="original">
        <TabsList className={cn("grid w-full", tabClass)}>
          <TabsTrigger value="original">{t("original")}</TabsTrigger>
          {different && <TabsTrigger value="full">{t("full")}</TabsTrigger>}
          <TabsTrigger value="import">{t("import")}</TabsTrigger>
        </TabsList>
        <TabsContent value="full">
          <Card className="p-4">
            <pre>{fullToml}</pre>
          </Card>
        </TabsContent>
        <TabsContent value="original">
          {originalToml && (
            <Card className="p-4">
              <Button
                className="absolute right-10"
                variant="ghost"
                size="icon"
                onClick={async (e) => {
                  e.preventDefault();
                  copyToml();
                }}
              >
                <ClipboardCopy />
              </Button>
              <pre>{originalToml}</pre>
            </Card>
          )}
        </TabsContent>
        <TabsContent value="import">
          <Card className="p-4">
            <Textarea
              autoFocus
              rows={25}
              onChange={(e) => {
                const value = e.target.value.trim();
                setNewToml(value);
              }}
            />
            <Button
              className="w-full mt-5"
              disabled={importing}
              onClick={() => {
                handleImportToml(newToml);
              }}
            >
              {importText}
            </Button>
          </Card>
        </TabsContent>
      </Tabs>
    </div>
  );
}
