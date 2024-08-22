import React from 'react';
import { invoke } from "@tauri-apps/api/core";
import { Input } from "./components/ui/input";
import { Button } from "./components/ui/button";
import {
    useQueryClient,
    useMutation,
} from "@tanstack/react-query";
import { useForm } from "@tanstack/react-form";

interface BlockInputProps {
    parent: any
}

const BlockInput: React.FC<BlockInputProps> = ({ parent }) => {

    const queryClient = useQueryClient();

    const mutation = useMutation({
        mutationFn: async (block: any) => {
            console.debug("Invoking block command for ", block)
            return await invoke(
                "create_block_command",
                {
                    rawData: block.blockContent,
                    blockType: block.blockType,
                    parentId: parent.id
                })
                .then((res) => {
                    let data = JSON.parse(res as string);
                    console.debug(data);
                    return data;
                });
        },
        onSuccess: (data) => {
            console.log("Mutation Success", data);
            queryClient.invalidateQueries({ queryKey: ["displayedBlock"] });
        }
    });

    const blockForm = useForm({
        defaultValues: {
            blockType: 'paragraph',
            blockContent: '',
        },
        onSubmit: async (values) => {
            console.log("Submitting form", values);
            mutation.mutate({ blockContent: values.value.blockContent, blockType: values.value.blockType });
            console.log("cleaning block input content")
            values.value.blockContent = "";
        }

    })

    return (
        <div>
            <form
                className="flex group/block-input-form"
                onSubmit={(e) => {
                    e.preventDefault();
                    e.stopPropagation();
                    blockForm.handleSubmit();
                }}
            >
                <blockForm.Field
                    name="blockContent"
                    children={(field) => (
                        <Input
                            name={field.name}
                            value={field.state.value}
                            onChange={(e) => field.handleChange(e.target.value)}
                            // onBlur={blockForm.handleSubmit()}
                            placeholder="..."
                        />
                    )}
                />
                <blockForm.Field
                    name="blockType"
                    children={(field) => (
                        <>
                            <Button className="hidden group-hover/block-input-form:block" value="paragraph" type="submit" onClick={() => field.handleChange("paragraph")}>
                                Insert Block
                            </Button>
                            <Button className="hidden group-hover/block-input-form:block" value="page" type="submit" onClick={() => field.handleChange("page")}>
                                Create Page
                            </Button>
                        </>
                    )}
                />
            </form>
        </div>
    )
}
// <Button className="hidden group-hover /block-input-form:block" type="submit" value="paragraph">Create Block</Button>
// <Button className="hidden group-hover/block-input-form:block" type="submit" value="page">Create Page</Button>

// <Input
//     onChange={(e) => setBlockContent(e.currentTarget.value)}
//     value={blockContent}
//     placeholder="..."
// onBlur={(e) => {
//     mutation.mutate(blockContent);
//     console.log("cleaning block input content")
//     setBlockContent("");
// }
// }
// />
export default BlockInput;